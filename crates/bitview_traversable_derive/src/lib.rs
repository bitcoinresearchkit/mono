use std::iter;

use proc_macro::TokenStream;
use proc_macro2::{Group, Span, TokenStream as ProcMacro2TokenStream, TokenTree};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, ExprLit, Field, Fields, GenericArgument,
    GenericParam, Generics, Ident, Index, Lit, Member, Meta, PathArguments, Result, Token, Type,
    TypeParam, WherePredicate, parse_macro_input, parse_quote, punctuated::Punctuated,
};

// ===========================================================================
// Attribute model, parsed once for both generators
// ===========================================================================

/// One field and its traversal options.
struct FieldModel<'a> {
    field: &'a Field,
    /// `self.<member>`: the field name, or `0` for a tuple struct.
    member: Member,
    /// Left out of the traversal: `skip`, or writer-only state (`M::WriteOnly<..>`).
    skipped: bool,
    flatten: bool,
    hidden: bool,
    rename: Option<String>,
    wrap: Option<String>,
    description: Option<String>,
}

impl FieldModel<'_> {
    fn ty(&self) -> &Type {
        &self.field.ty
    }

    fn has_options(&self) -> bool {
        self.skipped || self.flatten || self.hidden || self.rename.is_some() || self.wrap.is_some()
    }
}

/// The options of every `#[traversable(..)]` attribute in `attrs`.
fn traversable_metas(attrs: &[Attribute]) -> Result<Vec<Meta>> {
    let mut metas = Vec::new();
    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("traversable"))
    {
        metas.extend(attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?);
    }
    Ok(metas)
}

/// Whether the struct carries `#[traversable(merge)]`, its only option.
fn parse_merge(attrs: &[Attribute]) -> Result<bool> {
    let mut merge = false;
    for meta in traversable_metas(attrs)? {
        match &meta {
            Meta::Path(path) if path.is_ident("merge") => merge = true,
            _ => return Err(Error::new_spanned(meta, "expected `merge`")),
        }
    }
    Ok(merge)
}

fn parse_field(index: usize, field: &Field) -> Result<FieldModel<'_>> {
    let mut model = FieldModel {
        field,
        member: field
            .ident
            .clone()
            .map_or_else(|| Member::Unnamed(Index::from(index)), Member::Named),
        skipped: is_write_only_type(&field.ty),
        flatten: false,
        hidden: false,
        rename: None,
        wrap: None,
        description: get_doc_comment(field),
    };
    for meta in traversable_metas(&field.attrs)? {
        match &meta {
            Meta::Path(path) if path.is_ident("skip") => model.skipped = true,
            Meta::Path(path) if path.is_ident("flatten") => model.flatten = true,
            Meta::Path(path) if path.is_ident("hidden") => model.hidden = true,
            Meta::NameValue(name_value)
                if name_value.path.is_ident("rename") || name_value.path.is_ident("wrap") =>
            {
                let Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) = &name_value.value
                else {
                    return Err(Error::new_spanned(&name_value.value, "expected a string"));
                };
                let slot = if name_value.path.is_ident("rename") {
                    &mut model.rename
                } else {
                    &mut model.wrap
                };
                *slot = Some(value.value());
            }
            _ => {
                return Err(Error::new_spanned(
                    meta,
                    "expected `skip`, `flatten`, `hidden`, `rename = \"..\"` or `wrap = \"..\"`",
                ));
            }
        }
    }
    Ok(model)
}

/// Parses every field, rejecting shapes the traversal cannot express: enums, unit structs,
/// multi-field tuple structs, and options on a delegating tuple field.
fn parse_fields<'a>(input: &'a DeriveInput, merge: bool) -> Result<Vec<FieldModel<'a>>> {
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            &input.ident,
            "Traversable can only be derived for structs",
        ));
    };
    let fields = data
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| parse_field(index, field))
        .collect::<Result<Vec<_>>>()?;
    match &data.fields {
        Fields::Named(_) => Ok(fields),
        Fields::Unnamed(_) if fields.len() == 1 && !merge && !fields[0].has_options() => Ok(fields),
        _ => Err(Error::new_spanned(
            &input.ident,
            "Traversable needs named fields, or one tuple field it delegates to (without options)",
        )),
    }
}

fn get_doc_comment(field: &Field) -> Option<String> {
    let lines = field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| match &attr.meta {
            Meta::NameValue(meta) => match &meta.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) => Some(value.value()),
                _ => None,
            },
            _ => None,
        })
        .map(|line| line.trim().to_string())
        .collect::<Vec<_>>();

    (!lines.is_empty()).then(|| lines.join(" "))
}

fn with_description_fragment(
    description: Option<&str>,
    collect: ProcMacro2TokenStream,
) -> ProcMacro2TokenStream {
    let Some(description) = description else {
        return collect;
    };

    quote! {{
        description_fragments.push(#description);
        #collect
        let popped = description_fragments.pop();
        debug_assert_eq!(popped, Some(#description));
    }}
}

// ===========================================================================
// Type helpers
// ===========================================================================

/// Writer-only associated fields have no traversal or read-only payload.
fn is_write_only_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path)
        if (path.qself.is_some() || path.path.segments.len() > 1)
            && path.path.segments.last().is_some_and(|segment|
            segment.ident == "WriteOnly"
                && matches!(&segment.arguments, syn::PathArguments::AngleBracketed(args)
                    if args.args.len() == 1)))
}

/// The `T` of a `Box<T>` field type.
fn extract_box_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let seg = type_path.path.segments.last()?;
    if seg.ident != "Box" {
        return None;
    }
    match &seg.arguments {
        PathArguments::AngleBracketed(args) => args.args.iter().find_map(|arg| match arg {
            GenericArgument::Type(inner) => Some(inner),
            _ => None,
        }),
        _ => None,
    }
}

fn is_box_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Path(type_path)
        if type_path.path.segments.last()
            .is_some_and(|seg| seg.ident == "Box")
    )
}

/// Check if a type AST references the given identifier anywhere.
fn type_contains_ident(ty: &Type, ident: &Ident) -> bool {
    match ty {
        Type::Path(type_path) => {
            if let Some(qself) = &type_path.qself
                && type_contains_ident(&qself.ty, ident)
            {
                return true;
            }
            type_path.path.segments.iter().any(|seg| {
                if seg.ident == *ident {
                    return true;
                }
                match &seg.arguments {
                    PathArguments::AngleBracketed(args) => args.args.iter().any(|arg| {
                        matches!(arg, syn::GenericArgument::Type(inner) if type_contains_ident(inner, ident))
                    }),
                    PathArguments::Parenthesized(args) => {
                        args.inputs.iter().any(|inner| type_contains_ident(&inner.ty, ident))
                            || matches!(&args.output, syn::ReturnType::Type(_, inner) if type_contains_ident(inner, ident))
                    }
                    PathArguments::None => false,
                }
            })
        }
        Type::Reference(r) => type_contains_ident(&r.elem, ident),
        Type::Tuple(t) => t.elems.iter().any(|e| type_contains_ident(e, ident)),
        Type::Array(a) => type_contains_ident(&a.elem, ident),
        Type::Slice(s) => type_contains_ident(&s.elem, ident),
        Type::Paren(p) => type_contains_ident(&p.elem, ident),
        _ => false,
    }
}

/// Whether `ty` reaches through one of `params`, as in `G::Of<..>`: a generated
/// impl cannot see through such a projection without an explicit bound.
fn type_projects_param(ty: &Type, params: &[&Ident]) -> bool {
    match ty {
        Type::Path(type_path) => {
            let segments = &type_path.path.segments;
            type_path
                .qself
                .as_ref()
                .is_some_and(|q| params.iter().any(|p| type_contains_ident(&q.ty, p)))
                || (segments.len() > 1 && params.iter().any(|p| segments[0].ident == **p))
                || segments.iter().any(|seg| match &seg.arguments {
                    PathArguments::AngleBracketed(args) => args.args.iter().any(|arg| {
                        matches!(arg, GenericArgument::Type(inner) if type_projects_param(inner, params))
                    }),
                    _ => false,
                })
        }
        Type::Reference(r) => type_projects_param(&r.elem, params),
        Type::Tuple(t) => t.elems.iter().any(|e| type_projects_param(e, params)),
        Type::Array(a) => type_projects_param(&a.elem, params),
        Type::Slice(s) => type_projects_param(&s.elem, params),
        Type::Paren(p) => type_projects_param(&p.elem, params),
        _ => false,
    }
}

/// Replaces the storage-mode param `mode` in `tokens` with the concrete `to`,
/// qualifying projections such as `M::Stored<..>` as `<to as StorageMode>::..`.
fn substitute_mode(
    tokens: ProcMacro2TokenStream,
    mode: &Ident,
    to: &ProcMacro2TokenStream,
) -> ProcMacro2TokenStream {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    let mut out = ProcMacro2TokenStream::new();
    for (i, tree) in trees.iter().enumerate() {
        match tree {
            TokenTree::Ident(ident) if ident == mode => {
                let projects =
                    matches!(trees.get(i + 1), Some(TokenTree::Punct(p)) if p.as_char() == ':');
                out.extend(if projects {
                    quote! { <#to as bitview_traversable::StorageMode> }
                } else {
                    to.clone()
                });
            }
            TokenTree::Group(group) => {
                let mut substituted =
                    Group::new(group.delimiter(), substitute_mode(group.stream(), mode, to));
                substituted.set_span(group.span());
                out.extend([TokenTree::Group(substituted)]);
            }
            other => out.extend([other.clone()]),
        }
    }
    out
}

/// Find the generic type parameter bounded by `StorageMode`, if any.
fn find_storage_mode_param(generics: &Generics) -> Option<&Ident> {
    generics.type_params().find_map(|p| {
        p.bounds
            .iter()
            .any(|b| {
                matches!(b, syn::TypeParamBound::Trait(t)
                    if t.path.segments.last().is_some_and(|s| s.ident == "StorageMode"))
            })
            .then_some(&p.ident)
    })
}

// ===========================================================================
// Entry point
// ===========================================================================

#[proc_macro_derive(Traversable, attributes(traversable))]
pub fn derive_traversable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive(&input) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn derive(input: &DeriveInput) -> Result<ProcMacro2TokenStream> {
    let merge = parse_merge(&input.attrs)?;
    let fields = parse_fields(input, merge)?;
    let mut output = gen_traversable(input, &fields, merge);
    output.extend(gen_read_only_clone(input, &fields));
    Ok(output)
}

// ===========================================================================
// Traversable generation
// ===========================================================================

fn gen_traversable(
    input: &DeriveInput,
    fields: &[FieldModel],
    merge: bool,
) -> ProcMacro2TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, _) = generics.split_for_impl();

    // Single-field tuple struct: delegate.
    if let [field] = fields
        && let Member::Unnamed(_) = field.member
    {
        let where_clause = build_where_clause(generics, &[field.ty()]);
        let collect_description = with_description_fragment(
            field.description.as_deref(),
            quote! {
                self.0.collect_series_descriptions(description_fragments, descriptions);
            },
        );
        return quote! {
            impl #impl_generics Traversable for #name #ty_generics #where_clause {
                fn to_tree_node(&self) -> bitview_traversable::TreeNode {
                    self.0.to_tree_node()
                }

                fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                    self.0.iter_any_exportable()
                }

                fn iter_any_visible(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                    self.0.iter_any_visible()
                }

                fn iter_any_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn bitview_traversable::AnyStoredVec> {
                    self.0.iter_any_stored_mut()
                }

                fn collect_series_descriptions<'a>(
                    &'a self,
                    description_fragments: &mut Vec<&'static str>,
                    descriptions: &mut std::collections::BTreeMap<&'a str, Vec<&'static str>>,
                ) {
                    #collect_description
                }
            }
        };
    }

    let traversed: Vec<&FieldModel> = fields.iter().filter(|f| !f.skipped).collect();
    let visible: Vec<&FieldModel> = traversed.iter().copied().filter(|f| !f.hidden).collect();

    let traversed_types: Vec<&Type> = traversed.iter().map(|f| f.ty()).collect();
    let where_clause = build_where_clause(generics, &traversed_types);

    let to_tree_node_body = generate_field_traversals(&visible, merge);
    let exportable_vec = quote! { &dyn bitview_traversable::AnyExportableVec };
    let exportable = collect_iters(&traversed, "iter_any_exportable", &exportable_vec);
    let visible_iter = collect_iters(&visible, "iter_any_visible", &exportable_vec);
    let stored = collect_iters(
        &traversed,
        "iter_any_stored_mut",
        &quote! { &mut dyn bitview_traversable::AnyStoredVec },
    );
    let description_impl = generate_description_impl(&visible);

    quote! {
        impl #impl_generics Traversable for #name #ty_generics #where_clause {
            fn to_tree_node(&self) -> bitview_traversable::TreeNode {
                { #to_tree_node_body }
            }

            fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                #exportable
            }

            fn iter_any_visible(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                #visible_iter
            }

            fn iter_any_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn bitview_traversable::AnyStoredVec> {
                #stored
            }

            #description_impl
        }
    }
}

/// Bounds every traversed field type by `Traversable` and every type param by `Send + Sync`.
fn build_where_clause(generics: &Generics, traversed_types: &[&Type]) -> ProcMacro2TokenStream {
    let generic_params = generics.type_params().map(|p| &p.ident);
    let original_predicates = generics.where_clause.as_ref().map(|w| &w.predicates);
    quote! {
        where
            #(#traversed_types: bitview_traversable::Traversable,)*
            #(#generic_params: Send + Sync,)*
            #original_predicates
    }
}

/// The fields' vecs, collected into one `Vec`: compiles faster than iterator chains (boxed or
/// not) and keeps each node's iterator small, as deep trees need. These walks only serve
/// startup, the catalog and exports. A single field passes through unchanged.
fn collect_iters(
    fields: &[&FieldModel],
    method: &str,
    item: &ProcMacro2TokenStream,
) -> ProcMacro2TokenStream {
    let method = Ident::new(method, Span::call_site());
    let iters: Vec<_> = fields
        .iter()
        .map(|field| {
            let member = &field.member;
            quote! { self.#member.#method() }
        })
        .collect();
    match iters.as_slice() {
        [] => quote! { std::iter::empty() },
        [only] => quote! { #only },
        [first, rest @ ..] => quote! {
            let mut vecs: Vec<#item> = #first.collect();
            #(vecs.extend(#rest);)*
            vecs.into_iter()
        },
    }
}

fn generate_field_traversals(visible: &[&FieldModel], merge: bool) -> ProcMacro2TokenStream {
    // A struct whose only visible field is a flattened view is that view, a leaf included
    // (stored sources beside it stay hidden).
    if !merge
        && let [only] = visible
        && only.flatten
        && only.rename.is_none()
    {
        let member = &only.member;
        return quote! { self.#member.to_tree_node() };
    }

    // Process all fields in declaration order (interleaving normal and flatten)
    // so that struct field order determines tree key order.
    let field_operations = visible.iter().map(|info| {
        let member = &info.member;
        let Member::Named(field_name) = member else {
            unreachable!("only tuple structs have unnamed fields, and they delegate");
        };
        if info.flatten {
            // A flattened leaf keeps a key: its rename, else the field name.
            let leaf_key = info
                .rename
                .clone()
                .unwrap_or_else(|| field_name.to_string());
            return quote! {
                match self.#member.to_tree_node() {
                    bitview_traversable::TreeNode::Branch(map) => {
                        collected.merge_fields(map);
                    }
                    leaf @ bitview_traversable::TreeNode::Leaf(_) => {
                        collected.merge_field(String::from(#leaf_key), leaf);
                    }
                }
            };
        }

        let field_name_str = {
            let s = field_name.to_string();
            let s = s.strip_prefix("r#").unwrap_or(&s);
            s.strip_prefix('_').unwrap_or(s).to_string()
        };

        // Determine the tree key and optional wrapping path.
        // wrap = "a/b" means: outer_key = "a", wrap the node under "b" then under the rename/field name.
        // wrap = "a" means: outer_key = "a", wrap under rename or field name.
        // No wrap: outer_key = rename or field name, no wrapping.
        let key = info.rename.as_deref().unwrap_or(&field_name_str);
        let (outer_key, wrap_path) = match info.wrap.as_deref() {
            Some(wrap) => {
                let mut parts = wrap.split('/');
                let outer = parts.next().unwrap();
                let path = parts.chain(iter::once(key)).collect::<Vec<_>>();
                (outer, path)
            }
            None => (key, vec![]),
        };

        // Build nested wrapping: wrap(path[last], wrap(path[last-1], ... node))
        let build_wrapped = |base: ProcMacro2TokenStream| -> ProcMacro2TokenStream {
            wrap_path.iter().rev().fold(base, |inner, key| {
                quote! { bitview_traversable::TreeNode::wrap(#key, #inner) }
            })
        };

        let node_expr = build_wrapped(quote! { self.#member.to_tree_node() });
        quote! {
            collected.merge_field(String::from(#outer_key), #node_expr);
        }
    });

    let final_expr = if merge {
        quote! { bitview_traversable::TreeNode::Branch(collected).merge_branches() }
    } else {
        quote! { bitview_traversable::TreeNode::Branch(collected) }
    };

    quote! {
        let mut collected = bitview_traversable::TreeBranch::default();
        #(#field_operations)*
        #final_expr
    }
}

/// Collects each visible field's descriptions under its doc comment. Without visible fields
/// the trait's default (which visits no vec) applies.
fn generate_description_impl(visible: &[&FieldModel]) -> ProcMacro2TokenStream {
    if visible.is_empty() {
        return quote! {};
    }

    let fields = visible.iter().map(|info| {
        let member = &info.member;
        with_description_fragment(
            info.description.as_deref(),
            quote! {
                self.#member.collect_series_descriptions(description_fragments, descriptions);
            },
        )
    });

    quote! {
        fn collect_series_descriptions<'a>(
            &'a self,
            description_fragments: &mut Vec<&'static str>,
            descriptions: &mut std::collections::BTreeMap<&'a str, Vec<&'static str>>,
        ) {
            #(#fields)*
        }
    }
}

// ===========================================================================
// ReadOnlyClone generation
// ===========================================================================

/// Generate `ReadOnlyClone` for Traversable-derived types.
///
/// Three paths:
/// 1. `M: StorageMode` → concrete impl mapping `Self<Rw>` → `Self<Ro>`.
/// 2. Generic container params → propagates `ReadOnlyClone` through each param.
/// 3. No container params → nothing generated.
///
/// Container params are: unbounded type params, OR bounded params that appear
/// as a bare field type (e.g. `field: M` where M is the param itself).
fn gen_read_only_clone(input: &DeriveInput, fields: &[FieldModel]) -> ProcMacro2TokenStream {
    let generics = &input.generics;
    let name = &input.ident;

    // Path 1: StorageMode param → Rw/Ro substitution.
    if let Some(mode_param) = find_storage_mode_param(generics) {
        return gen_read_only_clone_storage_mode(name, generics, fields, mode_param);
    }

    // Path 2/3: classify type params as containers or leaves.
    let type_params: Vec<&TypeParam> = generics.type_params().collect();

    if type_params.is_empty() {
        return quote! {};
    }

    let is_bounded = |tp: &TypeParam| -> bool {
        if !tp.bounds.is_empty() {
            return true;
        }
        if let Some(wc) = &generics.where_clause {
            return wc.predicates.iter().any(|pred| {
                matches!(pred, syn::WherePredicate::Type(pt)
                    if matches!(&pt.bounded_ty, Type::Path(p)
                        if p.path.segments.first().is_some_and(|s| s.ident == tp.ident)))
            });
        }
        false
    };

    let bare_field_params = find_bare_field_params(fields, &type_params);

    let container_params: Vec<&Ident> = type_params
        .iter()
        .filter(|tp| !is_bounded(tp) || bare_field_params.contains(&&tp.ident))
        .map(|tp| &tp.ident)
        .collect();

    if container_params.is_empty() {
        return quote! {};
    }

    gen_read_only_clone_generics(name, generics, fields, &container_params)
}

/// Find type params used as bare (direct) field types in non-skipped fields.
fn find_bare_field_params<'a>(
    fields: &[FieldModel],
    type_params: &[&'a TypeParam],
) -> Vec<&'a Ident> {
    let mut bare = Vec::new();
    for field in fields.iter().filter(|field| !field.skipped) {
        if let Type::Path(type_path) = field.ty()
            && type_path.path.segments.len() == 1
            && let Some(seg) = type_path.path.segments.first()
            && seg.arguments.is_empty()
            && let Some(tp) = type_params.iter().find(|tp| tp.ident == seg.ident)
        {
            bare.push(&tp.ident);
        }
    }
    bare
}

// ---------------------------------------------------------------------------
// Shared field-conversion helpers
// ---------------------------------------------------------------------------

/// Generate the value expression for a single field in a ReadOnlyClone impl.
///
/// - `M::WriteOnly<T>` → `()`
/// - Skipped → `Clone::clone()`
/// - Contains relevant param + Box → `Box::new(read_only_clone(&*self.field))`
/// - Contains relevant param → `read_only_clone(&self.field)`
/// - Otherwise → `self.field.clone()`
fn gen_roc_field_value(
    field: &FieldModel,
    is_relevant: impl Fn(&Type) -> bool,
) -> ProcMacro2TokenStream {
    let member = &field.member;
    let ty = field.ty();
    if is_write_only_type(ty) {
        return quote! { () };
    }
    if field.skipped || !is_relevant(ty) {
        return quote! { self.#member.clone() };
    }
    if is_box_type(ty) {
        quote! { Box::new(bitview_traversable::ReadOnlyClone::read_only_clone(&*self.#member)) }
    } else {
        quote! { bitview_traversable::ReadOnlyClone::read_only_clone(&self.#member) }
    }
}

/// Generate the struct body for a ReadOnlyClone impl (`Name { 0: .. }` for a tuple struct).
fn gen_roc_body(
    name: &Ident,
    fields: &[FieldModel],
    is_relevant: impl Fn(&Type) -> bool,
) -> ProcMacro2TokenStream {
    let conversions = fields.iter().map(|field| {
        let member = &field.member;
        let value = gen_roc_field_value(field, &is_relevant);
        quote! { #member: #value }
    });
    quote! { #name { #(#conversions,)* } }
}

/// Collect type args from generics, applying a mapping function to each.
fn collect_ty_args(
    generics: &Generics,
    map_type: impl Fn(&TypeParam) -> ProcMacro2TokenStream,
) -> Vec<ProcMacro2TokenStream> {
    generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(tp) => map_type(tp),
            GenericParam::Lifetime(lt) => {
                let lt = &lt.lifetime;
                quote! { #lt }
            }
            GenericParam::Const(c) => {
                let id = &c.ident;
                quote! { #id }
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Path 1: StorageMode → Rw/Ro substitution
// ---------------------------------------------------------------------------

fn gen_read_only_clone_storage_mode(
    name: &Ident,
    generics: &Generics,
    fields: &[FieldModel],
    mode_param: &Ident,
) -> ProcMacro2TokenStream {
    let make_ty_args = |replacement: ProcMacro2TokenStream| {
        collect_ty_args(generics, |tp| {
            if tp.ident == *mode_param {
                replacement.clone()
            } else {
                let id = &tp.ident;
                quote! { #id }
            }
        })
    };

    let ty_args_rw = make_ty_args(quote! { bitview_traversable::Rw });
    let ty_args_ro = make_ty_args(quote! { bitview_traversable::Ro });

    // With other type params, the conversion of each field depending on them is
    // stated: fields holding `M` convert through `ReadOnlyClone`, projections
    // such as `G::Of<..>` clone.
    let other_params: Vec<&Ident> = generics
        .type_params()
        .map(|p| &p.ident)
        .filter(|ident| *ident != mode_param)
        .collect();
    let field_bounds = fields
        .iter()
        .filter(|f| !is_write_only_type(f.ty()))
        .filter_map(|f| -> Option<WherePredicate> {
            let ty = f.ty();
            if f.skipped {
                // Skipped fields are cloned.
                return type_projects_param(ty, &other_params).then(|| parse_quote! { #ty: Clone });
            }
            let generic = other_params.iter().any(|p| type_contains_ident(ty, p));
            if generic && type_contains_ident(ty, mode_param) {
                // Boxed fields convert their contents.
                let ty = extract_box_inner(ty).unwrap_or(ty);
                let rw = substitute_mode(
                    quote! { #ty },
                    mode_param,
                    &quote! { bitview_traversable::Rw },
                );
                let ro = substitute_mode(
                    quote! { #ty },
                    mode_param,
                    &quote! { bitview_traversable::Ro },
                );
                Some(parse_quote! { #rw: bitview_traversable::ReadOnlyClone<ReadOnly = #ro> })
            } else if type_projects_param(ty, &other_params) {
                Some(parse_quote! { #ty: Clone })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // The impl keeps every other param, its bounds and the struct's predicates.
    let mut impl_generics = generics.clone();
    impl_generics.params = generics
        .params
        .iter()
        .filter(|p| !matches!(p, GenericParam::Type(tp) if tp.ident == *mode_param))
        .cloned()
        .collect();
    impl_generics
        .make_where_clause()
        .predicates
        .extend(field_bounds);
    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();

    let body = gen_roc_body(name, fields, |ty| type_contains_ident(ty, mode_param));

    quote! {
        impl #impl_generics bitview_traversable::ReadOnlyClone for #name<#(#ty_args_rw),*> #where_clause {
            type ReadOnly = #name<#(#ty_args_ro),*>;

            fn read_only_clone(&self) -> Self::ReadOnly {
                #body
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Path 2: Generic container params → ReadOnlyClone propagation
// ---------------------------------------------------------------------------

fn gen_read_only_clone_generics(
    name: &Ident,
    generics: &Generics,
    fields: &[FieldModel],
    container_params: &[&Ident],
) -> ProcMacro2TokenStream {
    // Check if any non-skipped field actually uses a container param.
    let has_container_field = fields.iter().any(|f| {
        !f.skipped
            && container_params
                .iter()
                .any(|tp| type_contains_ident(f.ty(), tp))
    });

    if !has_container_field {
        return quote! {};
    }

    let is_container = |ident: &Ident| container_params.contains(&ident);

    // Containers gain `ReadOnlyClone`; the bounds of bounded ones (inline or in the where
    // clause) carry over to their `ReadOnly` projection.
    let mut impl_generics = generics.clone();
    let mut extra_where: Vec<WherePredicate> = Vec::new();
    for tp in impl_generics.type_params_mut() {
        if !is_container(&tp.ident) {
            continue;
        }
        let ident = &tp.ident;
        if !tp.bounds.is_empty() {
            let bounds = &tp.bounds;
            extra_where.push(parse_quote! {
                <#ident as bitview_traversable::ReadOnlyClone>::ReadOnly: #bounds
            });
        }
        tp.bounds
            .push(parse_quote!(bitview_traversable::ReadOnlyClone));
    }
    for pred in generics.where_clause.iter().flat_map(|w| &w.predicates) {
        if let WherePredicate::Type(pt) = pred
            && let Type::Path(tp) = &pt.bounded_ty
            && let Some(seg) = tp.path.segments.first()
            && is_container(&seg.ident)
        {
            let ident = &seg.ident;
            let bounds = &pt.bounds;
            extra_where.push(parse_quote! {
                <#ident as bitview_traversable::ReadOnlyClone>::ReadOnly: #bounds
            });
        }
    }
    impl_generics
        .make_where_clause()
        .predicates
        .extend(extra_where);
    let (impl_generics, ty_generics, where_clause) = impl_generics.split_for_impl();

    let ro_ty_args = collect_ty_args(generics, |tp| {
        let id = &tp.ident;
        if is_container(id) {
            quote! { <#id as bitview_traversable::ReadOnlyClone>::ReadOnly }
        } else {
            quote! { #id }
        }
    });

    let body = gen_roc_body(name, fields, |ty| {
        container_params
            .iter()
            .any(|tp| type_contains_ident(ty, tp))
    });

    quote! {
        impl #impl_generics bitview_traversable::ReadOnlyClone for #name #ty_generics #where_clause {
            type ReadOnly = #name<#(#ro_ty_args),*>;

            fn read_only_clone(&self) -> Self::ReadOnly {
                #body
            }
        }
    }
}
