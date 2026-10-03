use std::{collections::BTreeSet, iter};

use proc_macro::TokenStream;
use proc_macro2::{Group, Span, TokenStream as ProcMacro2TokenStream, TokenTree};
use quote::quote;
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Error, Expr, ExprLit, Field, Fields, FieldsNamed,
    GenericArgument, GenericParam, Generics, Ident, Index, Lit, Meta, MetaNameValue, PathArguments,
    Token, Type, TypeParam, WherePredicate, parse_macro_input, punctuated::Punctuated,
};

// ===========================================================================
// Struct & field attribute parsing
// ===========================================================================

#[derive(Default)]
struct StructAttr {
    merge: bool,
    transparent: bool,
    hidden: bool,
    field_suffixes: bool,
    wrap: Option<String>,
}

fn get_struct_attr(attrs: &[Attribute]) -> StructAttr {
    let mut result = StructAttr::default();
    for attr in attrs {
        if !attr.path().is_ident("traversable") {
            continue;
        }

        if let Ok(ident) = attr.parse_args::<Ident>() {
            match ident.to_string().as_str() {
                "merge" => result.merge = true,
                "transparent" => result.transparent = true,
                "hidden" => result.hidden = true,
                "field_suffixes" => result.field_suffixes = true,
                _ => {}
            }
            continue;
        }

        if let Ok(meta) = attr.parse_args::<MetaNameValue>()
            && meta.path.is_ident("wrap")
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) = &meta.value
        {
            result.wrap = Some(lit_str.value());
        }
    }
    result
}

enum FieldAttr {
    Normal,
    Flatten,
}

struct FieldInfo<'a> {
    name: &'a Ident,
    is_option: bool,
    attr: FieldAttr,
    rename: Option<String>,
    wrap: Option<String>,
    hidden: bool,
    description: Option<String>,
}

struct ParsedFieldAttr {
    attr: FieldAttr,
    rename: Option<String>,
    wrap: Option<String>,
    hidden: bool,
}

/// Returns `None` for skipped fields and parsed traversal metadata otherwise.
fn get_field_attr(field: &Field) -> Option<ParsedFieldAttr> {
    if is_write_only_type(&field.ty) {
        return None;
    }
    let mut attr_type = FieldAttr::Normal;
    let mut rename = None;
    let mut wrap = None;
    let mut hidden = false;
    for attr in &field.attrs {
        if !attr.path().is_ident("traversable") {
            continue;
        }

        let Ok(metas) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };

        for meta in metas {
            match meta {
                Meta::Path(path) if path.is_ident("skip") => return None,
                Meta::Path(path) if path.is_ident("flatten") => {
                    attr_type = FieldAttr::Flatten;
                }
                Meta::Path(path) if path.is_ident("hidden") => hidden = true,
                Meta::NameValue(meta) => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = &meta.value
                    {
                        if meta.path.is_ident("rename") {
                            rename = Some(lit_str.value());
                        } else if meta.path.is_ident("wrap") {
                            wrap = Some(lit_str.value());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(ParsedFieldAttr {
        attr: attr_type,
        rename,
        wrap,
        hidden,
    })
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
        // The pop must remain unconditional: putting it inside `debug_assert_eq!`
        // removes the state change from optimized builds.
        assert_eq!(description_fragments.pop(), Some(#description));
    }}
}

fn is_field_skipped(field: &Field) -> bool {
    get_field_attr(field).is_none()
}

// ===========================================================================
// Type helpers
// ===========================================================================

fn is_option_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Path(type_path)
        if type_path.path.segments.last()
            .is_some_and(|seg| seg.ident == "Option")
    )
}

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

fn is_phantom_data_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Path(type_path)
        if type_path.path.segments.last()
            .is_some_and(|segment| segment.ident == "PhantomData")
    )
}

/// Extract the inner type from `Option<T>`, returning `Some(&T)`.
fn extract_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
        && seg.ident == "Option"
        && let PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(GenericArgument::Type(inner)) = args.args.first()
    {
        Some(inner)
    } else {
        None
    }
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

    let mut output = gen_traversable(&input);
    output.extend(gen_read_only_clone(&input));
    TokenStream::from(output)
}

// ===========================================================================
// Traversable generation
// ===========================================================================

fn gen_traversable(input: &DeriveInput) -> ProcMacro2TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, _) = generics.split_for_impl();

    let struct_attr = get_struct_attr(&input.attrs);

    let Data::Struct(data) = &input.data else {
        return Error::new_spanned(&input.ident, "Traversable can only be derived for structs")
            .to_compile_error();
    };

    // Single-field tuple struct: delegate (automatic transparent).
    if let Fields::Unnamed(fields) = &data.fields
        && fields.unnamed.len() == 1
        && !is_write_only_type(&fields.unnamed[0].ty)
    {
        let field = fields.unnamed.first().unwrap();
        let field_ty = &field.ty;
        let where_clause = build_where_clause(generics, &[], &[field_ty]);
        let collect_description = with_description_fragment(
            get_doc_comment(field).as_deref(),
            quote! {
                self.0.collect_series_descriptions(description_fragments, descriptions);
            },
        );
        let to_tree_node_body = if let Some(wrap_key) = &struct_attr.wrap {
            quote! { bitview_traversable::TreeNode::wrap(#wrap_key, self.0.to_tree_node()) }
        } else {
            quote! { self.0.to_tree_node() }
        };
        return quote! {
            impl #impl_generics Traversable for #name #ty_generics #where_clause {
                fn to_tree_node(&self) -> bitview_traversable::TreeNode {
                    { #to_tree_node_body }
                }

                fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                    self.0.iter_any_exportable()
                }

                fn iter_any_visible(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                    self.0.iter_any_visible()
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

    // Named fields required from here.
    let Fields::Named(named_fields) = &data.fields else {
        return quote! {
            impl #impl_generics Traversable for #name #ty_generics {
                fn to_tree_node(&self) -> bitview_traversable::TreeNode {
                    bitview_traversable::TreeNode::branch(bitview_traversable::IndexMap::new())
                }

                fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                    std::iter::empty()
                }

                fn collect_series_descriptions<'a>(
                    &'a self,
                    _description_fragments: &mut Vec<&'static str>,
                    _descriptions: &mut std::collections::BTreeMap<&'a str, Vec<&'static str>>,
                ) {}
            }
        };
    };

    // Transparent delegation: writer-only fields never participate.
    if struct_attr.transparent
        && let Some(first_field) = named_fields
            .named
            .iter()
            .find(|field| !is_write_only_type(&field.ty))
    {
        let field_name = first_field
            .ident
            .as_ref()
            .expect("named field must have ident");
        let field_ty = &first_field.ty;
        let where_clause = build_where_clause(generics, &[], &[field_ty]);
        let collect_description = with_description_fragment(
            get_doc_comment(first_field).as_deref(),
            quote! {
                self.#field_name.collect_series_descriptions(description_fragments, descriptions);
            },
        );
        return quote! {
            impl #impl_generics Traversable for #name #ty_generics #where_clause {
                fn to_tree_node(&self) -> bitview_traversable::TreeNode {
                    self.#field_name.to_tree_node()
                }

                fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                    self.#field_name.iter_any_exportable()
                }

                fn iter_any_visible(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                    self.#field_name.iter_any_visible()
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

    let generic_params: Vec<_> = generics.type_params().map(|p| &p.ident).collect();

    let (field_infos, generics_needing_traversable, field_traversable_types) =
        analyze_fields(named_fields, &generic_params);

    let field_traversals = generate_field_traversals(&field_infos, struct_attr.merge);
    let iterator_impl = generate_iterator_impl(&field_infos, struct_attr.hidden);
    let description_impl = generate_description_impl(&field_infos, struct_attr.hidden);
    let where_clause = build_where_clause(
        generics,
        &generics_needing_traversable,
        &field_traversable_types,
    );

    let to_tree_node_body = if struct_attr.hidden {
        quote! { bitview_traversable::TreeNode::branch(bitview_traversable::IndexMap::new()) }
    } else {
        field_traversals
    };

    let to_tree_node_body = if struct_attr.field_suffixes {
        quote! { { #to_tree_node_body }.with_field_suffixes() }
    } else {
        to_tree_node_body
    };

    quote! {
        impl #impl_generics Traversable for #name #ty_generics #where_clause {
            fn to_tree_node(&self) -> bitview_traversable::TreeNode {
                { #to_tree_node_body }
            }

            #iterator_impl
            #description_impl
        }
    }
}

fn analyze_fields<'a>(
    fields: &'a FieldsNamed,
    generic_params: &[&'a Ident],
) -> (Vec<FieldInfo<'a>>, Vec<&'a Ident>, Vec<&'a Type>) {
    let mut field_infos = Vec::new();
    let mut generics_set = BTreeSet::new();
    let mut field_traversable_types = Vec::new();

    for field in &fields.named {
        let Some(parsed) = get_field_attr(field) else {
            continue;
        };

        let Some(field_name) = &field.ident else {
            continue;
        };

        let is_option = is_option_type(&field.ty);

        if let Type::Path(type_path) = &field.ty
            && type_path.path.segments.len() == 1
            && let Some(seg) = type_path.path.segments.first()
            && seg.arguments.is_empty()
            && let Some(&param) = generic_params.iter().find(|&&g| g == &seg.ident)
        {
            generics_set.insert(param);
        } else {
            let ty = if is_option {
                extract_option_inner(&field.ty).unwrap_or(&field.ty)
            } else {
                &field.ty
            };
            field_traversable_types.push(ty);
        }

        field_infos.push(FieldInfo {
            name: field_name,
            is_option,
            attr: parsed.attr,
            rename: parsed.rename,
            wrap: parsed.wrap,
            hidden: parsed.hidden,
            description: get_doc_comment(field),
        });
    }

    (
        field_infos,
        generics_set.into_iter().collect(),
        field_traversable_types,
    )
}

fn build_where_clause(
    generics: &Generics,
    generics_needing_traversable: &[&Ident],
    extra_traversable_types: &[&Type],
) -> ProcMacro2TokenStream {
    let generic_params: Vec<_> = generics.type_params().map(|p| &p.ident).collect();
    let original_predicates = generics.where_clause.as_ref().map(|w| &w.predicates);

    if generics_needing_traversable.is_empty()
        && extra_traversable_types.is_empty()
        && generic_params.is_empty()
        && original_predicates.is_none()
    {
        return quote! {};
    }

    quote! {
        where
            #(#generics_needing_traversable: bitview_traversable::Traversable,)*
            #(#extra_traversable_types: bitview_traversable::Traversable,)*
            #(#generic_params: Send + Sync,)*
            #original_predicates
    }
}

fn generate_field_traversals(infos: &[FieldInfo], merge: bool) -> ProcMacro2TokenStream {
    // Process all fields in declaration order (interleaving normal and flatten)
    // so that struct field order determines tree key order.
    let field_operations: Vec<_> = infos
        .iter()
        .filter(|i| !i.hidden)
        .map(|info| {
            match info.attr {
                FieldAttr::Normal => {
                    let field_name = info.name;
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

                    if info.is_option {
                        let node_expr = build_wrapped(quote! { nested.to_tree_node() });
                        quote! {
                            if let Some(nested) = self.#field_name.as_ref() {
                                collected.merge_field(String::from(#outer_key), #node_expr);
                            }
                        }
                    } else {
                        let node_expr_self =
                            build_wrapped(quote! { self.#field_name.to_tree_node() });
                        quote! {
                            collected.merge_field(String::from(#outer_key), #node_expr_self);
                        }
                    }
                }
                FieldAttr::Flatten => {
                    let field_name = info.name;
                    let merge_branch = quote! {
                        bitview_traversable::TreeNode::Branch(map) => {
                            collected.merge_fields(map);
                        }
                        leaf @ bitview_traversable::TreeNode::Leaf(_) => {
                            collected.merge_field(String::from(stringify!(#field_name)), leaf);
                        }
                    };

                    if info.is_option {
                        quote! {
                            if let Some(ref nested) = self.#field_name {
                                match nested.to_tree_node() { #merge_branch }
                            }
                        }
                    } else {
                        quote! {
                            match self.#field_name.to_tree_node() { #merge_branch }
                        }
                    }
                }
            }
        })
        .collect();

    let final_expr = if merge {
        quote! { bitview_traversable::TreeNode::Branch(collected).merge_branches() }
    } else {
        quote! { bitview_traversable::TreeNode::Branch(collected) }
    };

    let init_collected = quote! {
        let mut collected = bitview_traversable::TreeBranch::default();
    };

    quote! {
        #init_collected
        #(#field_operations)*
        #final_expr
    }
}

fn generate_iter_body(
    fields: &[&Ident],
    option_fields: &[&Ident],
    method: &str,
) -> ProcMacro2TokenStream {
    let method_ident = Ident::new(method, Span::call_site());

    if fields.is_empty() && option_fields.is_empty() {
        return quote! { std::iter::empty() };
    }

    let (init_part, chain_part) = if let Some((&first, rest)) = fields.split_first() {
        (
            quote! {
                let mut iter: Box<dyn Iterator<Item = &dyn bitview_traversable::AnyExportableVec>> =
                    Box::new(self.#first.#method_ident());
            },
            quote! {
                #(iter = Box::new(iter.chain(self.#rest.#method_ident()));)*
            },
        )
    } else {
        (
            quote! {
                let mut iter: Box<dyn Iterator<Item = &dyn bitview_traversable::AnyExportableVec>> =
                    Box::new(std::iter::empty());
            },
            quote! {},
        )
    };

    let option_part = if !option_fields.is_empty() {
        let chains = option_fields.iter().map(|f| {
            quote! {
                if let Some(ref x) = self.#f {
                    iter = Box::new(iter.chain(x.#method_ident()));
                }
            }
        });
        quote! { #(#chains)* }
    } else {
        quote! {}
    };

    quote! {
        #init_part
        #chain_part
        #option_part
        iter
    }
}

fn generate_iterator_impl(infos: &[FieldInfo], struct_hidden: bool) -> ProcMacro2TokenStream {
    let all_regular: Vec<_> = infos
        .iter()
        .filter(|i| !i.is_option)
        .map(|i| i.name)
        .collect();
    let all_option: Vec<_> = infos
        .iter()
        .filter(|i| i.is_option)
        .map(|i| i.name)
        .collect();

    let exportable_body = generate_iter_body(&all_regular, &all_option, "iter_any_exportable");

    let visible_impl = if struct_hidden {
        // Entire struct is hidden — iter_any_visible returns nothing
        quote! {
            fn iter_any_visible(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                std::iter::empty()
            }
        }
    } else {
        // Always generate iter_any_visible that calls iter_any_visible on children
        // (skipping hidden fields if any), so hidden propagates through the tree
        let visible_regular: Vec<_> = infos
            .iter()
            .filter(|i| !i.is_option && !i.hidden)
            .map(|i| i.name)
            .collect();
        let visible_option: Vec<_> = infos
            .iter()
            .filter(|i| i.is_option && !i.hidden)
            .map(|i| i.name)
            .collect();
        let visible_body =
            generate_iter_body(&visible_regular, &visible_option, "iter_any_visible");
        quote! {
            fn iter_any_visible(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
                #visible_body
            }
        }
    };

    quote! {
        fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn bitview_traversable::AnyExportableVec> {
            #exportable_body
        }
        #visible_impl
    }
}

fn generate_description_impl(infos: &[FieldInfo], struct_hidden: bool) -> ProcMacro2TokenStream {
    if struct_hidden {
        return quote! {
            fn collect_series_descriptions<'a>(
                &'a self,
                _description_fragments: &mut Vec<&'static str>,
                _descriptions: &mut std::collections::BTreeMap<&'a str, Vec<&'static str>>,
            ) {}
        };
    }

    let fields = infos.iter().filter(|info| !info.hidden).map(|info| {
        let field_name = info.name;
        let collect = if info.is_option {
            quote! {
                if let Some(field) = &self.#field_name {
                    field.collect_series_descriptions(description_fragments, descriptions);
                }
            }
        } else {
            quote! {
                self.#field_name
                    .collect_series_descriptions(description_fragments, descriptions);
            }
        };

        with_description_fragment(info.description.as_deref(), collect)
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
fn gen_read_only_clone(input: &DeriveInput) -> ProcMacro2TokenStream {
    let generics = &input.generics;
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return quote! {};
    };

    // Path 1: StorageMode param → Rw/Ro substitution.
    if let Some(mode_param) = find_storage_mode_param(generics) {
        return gen_read_only_clone_storage_mode(name, generics, data, mode_param);
    }

    // Path 2/3: classify type params as containers or leaves.
    let type_params: Vec<&TypeParam> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => Some(tp),
            _ => None,
        })
        .collect();

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

    let bare_field_params = find_bare_field_params(data, &type_params);

    let container_params: Vec<&Ident> = type_params
        .iter()
        .filter(|tp| !is_bounded(tp) || bare_field_params.contains(&&tp.ident))
        .map(|tp| &tp.ident)
        .collect();

    if container_params.is_empty() {
        return quote! {};
    }

    gen_read_only_clone_generics(name, generics, data, &type_params, &container_params)
}

/// Find type params used as bare (direct) field types in non-skipped fields.
fn find_bare_field_params<'a>(data: &DataStruct, type_params: &[&'a TypeParam]) -> Vec<&'a Ident> {
    let fields: &Punctuated<Field, _> = match &data.fields {
        Fields::Named(named) => &named.named,
        Fields::Unnamed(unnamed) => &unnamed.unnamed,
        Fields::Unit => return Vec::new(),
    };

    let mut bare = Vec::new();
    for field in fields {
        if is_field_skipped(field) {
            continue;
        }
        if let Type::Path(type_path) = &field.ty
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
/// - Skipped + Option → `None`
/// - Skipped + non-Option → `Clone::clone()`
/// - Contains relevant param + Box → `Box::new(read_only_clone(&*self.field))`
/// - Contains relevant param → `read_only_clone(&self.field)`
/// - Otherwise → `self.field.clone()`
fn gen_roc_field_value(
    field: &Field,
    self_access: ProcMacro2TokenStream,
    is_relevant: impl Fn(&Type) -> bool,
) -> ProcMacro2TokenStream {
    if is_write_only_type(&field.ty) {
        return quote! { () };
    }
    if is_field_skipped(field) {
        if is_option_type(&field.ty) {
            return quote! { None };
        }
        if is_phantom_data_type(&field.ty) {
            return quote! { std::marker::PhantomData };
        }
        return quote! { #self_access.clone() };
    }

    if is_relevant(&field.ty) {
        if is_box_type(&field.ty) {
            quote! { Box::new(bitview_traversable::ReadOnlyClone::read_only_clone(&*#self_access)) }
        } else {
            quote! { bitview_traversable::ReadOnlyClone::read_only_clone(&#self_access) }
        }
    } else {
        quote! { #self_access.clone() }
    }
}

/// Generate the struct body for a ReadOnlyClone impl.
fn gen_roc_body(
    name: &Ident,
    data: &DataStruct,
    is_relevant: impl Fn(&Type) -> bool,
) -> ProcMacro2TokenStream {
    match &data.fields {
        Fields::Named(named) => {
            let conversions: Vec<_> = named
                .named
                .iter()
                .map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    let value = gen_roc_field_value(f, quote! { self.#field_name }, &is_relevant);
                    quote! { #field_name: #value }
                })
                .collect();
            quote! { #name { #(#conversions,)* } }
        }
        Fields::Unnamed(unnamed) => {
            let conversions: Vec<_> = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let idx = Index::from(i);
                    gen_roc_field_value(f, quote! { self.#idx }, &is_relevant)
                })
                .collect();
            quote! { #name(#(#conversions,)*) }
        }
        Fields::Unit => quote! { #name },
    }
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
    data: &DataStruct,
    mode_param: &Ident,
) -> ProcMacro2TokenStream {
    let impl_params: Vec<ProcMacro2TokenStream> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) if tp.ident == *mode_param => None,
            GenericParam::Type(tp) => {
                let ident = &tp.ident;
                let bounds = &tp.bounds;
                if bounds.is_empty() {
                    Some(quote! { #ident })
                } else {
                    Some(quote! { #ident: #bounds })
                }
            }
            GenericParam::Lifetime(lt) => Some(quote! { #lt }),
            GenericParam::Const(c) => {
                let ident = &c.ident;
                let ty = &c.ty;
                Some(quote! { const #ident: #ty })
            }
        })
        .collect();

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
    let fields: Vec<&Field> = match &data.fields {
        Fields::Named(named) => named.named.iter().collect(),
        Fields::Unnamed(unnamed) => unnamed.unnamed.iter().collect(),
        Fields::Unit => Vec::new(),
    };
    let field_bounds = fields
        .into_iter()
        .filter(|f| !is_write_only_type(&f.ty))
        .filter_map(|f| {
            let ty = &f.ty;
            if is_field_skipped(f) {
                // Skipped fields are cloned unless they reset to `None`/`PhantomData`.
                let cloned = !is_option_type(ty) && !is_phantom_data_type(ty);
                return (cloned && type_projects_param(ty, &other_params))
                    .then(|| quote! { #ty: Clone });
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
                Some(quote! { #rw: bitview_traversable::ReadOnlyClone<ReadOnly = #ro> })
            } else if type_projects_param(ty, &other_params) {
                Some(quote! { #ty: Clone })
            } else {
                None
            }
        });
    let predicates: Vec<ProcMacro2TokenStream> = generics
        .where_clause
        .iter()
        .flat_map(|w| w.predicates.iter().map(|p| quote! { #p }))
        .chain(field_bounds)
        .collect();
    let where_clause = if predicates.is_empty() {
        quote! {}
    } else {
        quote! { where #(#predicates),* }
    };

    let body = gen_roc_body(name, data, |ty| type_contains_ident(ty, mode_param));

    let impl_generics = if impl_params.is_empty() {
        quote! {}
    } else {
        quote! { <#(#impl_params),*> }
    };

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
    data: &DataStruct,
    type_params: &[&TypeParam],
    container_params: &[&Ident],
) -> ProcMacro2TokenStream {
    // Check if any non-skipped field actually uses a container param.
    let has_container_field = match &data.fields {
        Fields::Named(named) => named.named.iter().any(|f| {
            !is_field_skipped(f)
                && container_params
                    .iter()
                    .any(|tp| type_contains_ident(&f.ty, tp))
        }),
        Fields::Unnamed(unnamed) => unnamed.unnamed.iter().any(|f| {
            !is_field_skipped(f)
                && container_params
                    .iter()
                    .any(|tp| type_contains_ident(&f.ty, tp))
        }),
        Fields::Unit => false,
    };

    if !has_container_field {
        return quote! {};
    }

    let is_container = |ident: &Ident| container_params.contains(&ident);

    // Impl params: containers get ReadOnlyClone (+ original bounds), others keep their bounds.
    let impl_params: Vec<ProcMacro2TokenStream> = generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(tp) => {
                let ident = &tp.ident;
                let bounds = &tp.bounds;
                if is_container(ident) {
                    if bounds.is_empty() {
                        quote! { #ident: bitview_traversable::ReadOnlyClone }
                    } else {
                        quote! { #ident: #bounds + bitview_traversable::ReadOnlyClone }
                    }
                } else if bounds.is_empty() {
                    quote! { #ident }
                } else {
                    quote! { #ident: #bounds }
                }
            }
            GenericParam::Lifetime(lt) => quote! { #lt },
            GenericParam::Const(c) => {
                let ident = &c.ident;
                let ty = &c.ty;
                quote! { const #ident: #ty }
            }
        })
        .collect();

    let self_ty_args = collect_ty_args(generics, |tp| {
        let id = &tp.ident;
        quote! { #id }
    });

    let ro_ty_args = collect_ty_args(generics, |tp| {
        let id = &tp.ident;
        if is_container(id) {
            quote! { <#id as bitview_traversable::ReadOnlyClone>::ReadOnly }
        } else {
            quote! { #id }
        }
    });

    // Where clause: propagate bounds from bounded container params to their ReadOnly.
    let mut extra_where: Vec<ProcMacro2TokenStream> = Vec::new();

    for tp in type_params {
        if is_container(&tp.ident) && !tp.bounds.is_empty() {
            let ident = &tp.ident;
            let bounds = &tp.bounds;
            extra_where.push(quote! {
                <#ident as bitview_traversable::ReadOnlyClone>::ReadOnly: #bounds
            });
        }
    }

    if let Some(wc) = &generics.where_clause {
        for pred in &wc.predicates {
            if let WherePredicate::Type(pt) = pred
                && let Type::Path(tp) = &pt.bounded_ty
                && let Some(seg) = tp.path.segments.first()
                && container_params.iter().any(|cp| **cp == seg.ident)
            {
                let ident = &seg.ident;
                let bounds = &pt.bounds;
                extra_where.push(quote! {
                    <#ident as bitview_traversable::ReadOnlyClone>::ReadOnly: #bounds
                });
            }
        }
    }

    let original_predicates = generics.where_clause.as_ref().map(|w| &w.predicates);
    let combined_where = if extra_where.is_empty() && original_predicates.is_none() {
        quote! {}
    } else {
        quote! { where #(#extra_where,)* #original_predicates }
    };

    let body = gen_roc_body(name, data, |ty| {
        container_params
            .iter()
            .any(|tp| type_contains_ident(ty, tp))
    });

    quote! {
        impl<#(#impl_params),*> bitview_traversable::ReadOnlyClone for #name<#(#self_ty_args),*> #combined_where {
            type ReadOnly = #name<#(#ro_ty_args),*>;

            fn read_only_clone(&self) -> Self::ReadOnly {
                #body
            }
        }
    }
}
