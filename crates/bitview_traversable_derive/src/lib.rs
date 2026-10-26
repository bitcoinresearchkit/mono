use std::iter;

use proc_macro::TokenStream;
use proc_macro2::{Group, Spacing, Span, TokenStream as ProcMacro2TokenStream, TokenTree};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, ExprLit, Field, Fields, GenericArgument,
    GenericParam, Generics, Ident, Index, Lit, LitStr, Member, Meta, PathArguments, Result, Token,
    Type, TypeParam, WherePredicate, parse_macro_input, parse_quote, punctuated::Punctuated,
};

struct FieldModel<'a> {
    ty: &'a Type,
    /// `self.<member>`: the field name, or `0` for a tuple struct.
    member: Member,
    /// Left out of the traversal: `skip`, or writer-only state (`M::WriteOnly<..>`).
    skipped: bool,
    flatten: bool,
    hidden: bool,
    rename: Option<String>,
    wrap: Option<String>,
    /// A few plain words for anyone, opening the description of every series under the field.
    lead: Option<String>,
    description: Option<String>,
}

impl FieldModel<'_> {
    fn has_options(&self) -> bool {
        self.skipped || self.flatten || self.hidden || self.rename.is_some() || self.wrap.is_some()
    }
}

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

/// A struct's own options: `merge`, and `client = ".."`, the name client types take for it where
/// its Rust name describes storage (`LazyPercentCumulativeRolling` is `Share`).
fn parse_struct_options(attrs: &[Attribute]) -> Result<(bool, Option<LitStr>)> {
    let mut merge = false;
    let mut client = None;
    for meta in traversable_metas(attrs)? {
        match &meta {
            Meta::Path(path) if path.is_ident("merge") => merge = true,
            Meta::NameValue(name_value) if name_value.path.is_ident("client") => {
                let Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) = &name_value.value
                else {
                    return Err(Error::new_spanned(&name_value.value, "expected a string"));
                };
                if !is_type_name(&value.value()) {
                    return Err(Error::new_spanned(
                        value,
                        "a client name is a type name: ASCII letters and digits, an uppercase first",
                    ));
                }
                client = Some(value.clone());
            }
            _ => {
                return Err(Error::new_spanned(
                    meta,
                    "expected `merge` or `client = \"..\"`",
                ));
            }
        }
    }
    Ok((merge, client))
}

/// ASCII letters and digits, an uppercase first: valid in every client language, and clear of
/// their keywords, which are lowercase.
fn is_type_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_uppercase())
        && name.chars().all(|c| c.is_ascii_alphanumeric())
}

fn parse_field(index: usize, field: &Field) -> Result<FieldModel<'_>> {
    let mut model = FieldModel {
        ty: &field.ty,
        member: field
            .ident
            .clone()
            .map_or_else(|| Member::Unnamed(Index::from(index)), Member::Named),
        skipped: is_write_only_type(&field.ty),
        flatten: false,
        hidden: false,
        rename: None,
        wrap: None,
        lead: None,
        description: doc_comment(field),
    };
    for meta in traversable_metas(&field.attrs)? {
        match &meta {
            Meta::Path(path) if path.is_ident("skip") => model.skipped = true,
            Meta::Path(path) if path.is_ident("flatten") => model.flatten = true,
            Meta::Path(path) if path.is_ident("hidden") => model.hidden = true,
            Meta::NameValue(name_value)
                if name_value.path.is_ident("rename")
                    || name_value.path.is_ident("wrap")
                    || name_value.path.is_ident("lead") =>
            {
                // (A string a declarative macro passed in arrives in an invisible group.)
                let mut value = &name_value.value;
                while let Expr::Group(group) = value {
                    value = &group.expr;
                }
                let Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) = value
                else {
                    return Err(Error::new_spanned(&name_value.value, "expected a string"));
                };
                let slot = if name_value.path.is_ident("rename") {
                    &mut model.rename
                } else if name_value.path.is_ident("wrap") {
                    &mut model.wrap
                } else {
                    &mut model.lead
                };
                *slot = Some(value.value());
            }
            _ => {
                return Err(Error::new_spanned(
                    meta,
                    "expected `skip`, `flatten`, `hidden`, `rename = \"..\"`, `wrap = \"..\"` or `lead = \"..\"`",
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

/// A field's doc comment, as the Markdown it is: each paragraph's lines (wrapped where the code
/// was) joined into one, and the lines Markdown reads apart kept apart (a list's items, a quote's,
/// a table's rows, a heading, indented code); a fenced block as written; paragraphs and blocks a
/// blank line apart.
fn doc_comment(field: &Field) -> Option<String> {
    // (Each `///` line is an attribute of its own, a blank one empty; a block comment's holds its
    // lines.)
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
        .flat_map(|text| text.split('\n').map(str::to_owned).collect::<Vec<_>>());
    let mut markdown = String::new();
    // (The open fence's backticks, inside a fenced block: closed by as many or more.)
    let mut fence: Option<usize> = None;
    // (The last line one that takes no other on it: a heading, a table's row, indented code.)
    let mut closed = false;
    for line in lines {
        // (`/// text` is " text": the space after the slashes isn't the text's.)
        let raw = line.strip_prefix(' ').unwrap_or(&line);
        let line = raw.trim_end();
        let text = line.trim_start();
        let ticks = text.bytes().take_while(|&byte| byte == b'`').count();
        let indented = line.starts_with("    ");
        if let Some(open) = fence {
            markdown.push('\n');
            if ticks >= open && text[ticks..].trim().is_empty() {
                markdown.push_str(line);
                fence = None;
                apart(&mut markdown);
            } else {
                markdown.push_str(raw);
            }
            continue;
        }
        if ticks >= 3 {
            apart(&mut markdown);
            markdown.push_str(line);
            fence = Some(ticks);
        } else if text.is_empty() {
            apart(&mut markdown);
        } else if markdown.is_empty() || markdown.ends_with("\n\n") {
            markdown.push_str(if indented { line } else { text });
        } else if indented || starts_block(text) {
            markdown.push('\n');
            markdown.push_str(line);
        } else if closed {
            apart(&mut markdown);
            markdown.push_str(text);
        } else {
            markdown.push(' ');
            markdown.push_str(text);
        }
        closed = indented || text.starts_with('#') || text.starts_with('|');
    }
    let markdown = markdown.trim_end();
    (!markdown.is_empty()).then(|| markdown.to_owned())
}

/// What follows starts a paragraph or a block of its own: a blank line before it.
fn apart(markdown: &mut String) {
    if !markdown.is_empty() && !markdown.ends_with("\n\n") {
        markdown.push_str("\n\n");
    }
}

/// Whether a Markdown line starts a block of its own: a list's item, a quote, a table's row, a
/// heading.
fn starts_block(text: &str) -> bool {
    let ordered = text
        .split_once(". ")
        .is_some_and(|(number, _)| !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()));
    ordered || ["- ", "* ", "+ ", "> ", "|", "#"].iter().any(|start| text.starts_with(start))
}

/// `collect` with the field's description fragments on the path, its lead's and its doc
/// comment's, as long as it runs.
fn with_description_fragment(field: &FieldModel, collect: ProcMacro2TokenStream) -> ProcMacro2TokenStream {
    let fragments = field
        .lead
        .as_deref()
        .map(|lead| quote! { bitview_traversable::Fragment::Lead(#lead) })
        .into_iter()
        .chain(field.description.as_deref().map(|detail| quote! { bitview_traversable::Fragment::Detail(#detail) }))
        .collect::<Vec<_>>();
    if fragments.is_empty() {
        return collect;
    }
    let expected = fragments.iter().rev();

    quote! {{
        #(description_fragments.push(#fragments);)*
        #collect
        #(
            let popped = description_fragments.pop();
            debug_assert_eq!(popped, Some(#expected));
        )*
    }}
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
                // `M::..` (a joint `:`), not the `M: ..` of a bound.
                let projects = matches!(trees.get(i + 1), Some(TokenTree::Punct(p))
                    if p.as_char() == ':' && p.spacing() == Spacing::Joint);
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

/// Whether `tokens` name `ident` anywhere, nested groups included.
fn tokens_contain_ident(tokens: &ProcMacro2TokenStream, ident: &Ident) -> bool {
    tokens.clone().into_iter().any(|tree| match tree {
        TokenTree::Ident(i) => i == *ident,
        TokenTree::Group(group) => tokens_contain_ident(&group.stream(), ident),
        _ => false,
    })
}

fn is_maybe_bound(bound: &syn::TypeParamBound) -> bool {
    matches!(bound, syn::TypeParamBound::Trait(t) if t.maybe.is_some())
}

fn is_storage_mode_bound(bound: &syn::TypeParamBound) -> bool {
    matches!(bound, syn::TypeParamBound::Trait(t)
        if t.path.segments.last().is_some_and(|s| s.ident == "StorageMode"))
}

/// Find the generic type parameter bounded by `StorageMode` (inline or in the where clause).
fn find_storage_mode_param(generics: &Generics) -> Option<&Ident> {
    let is_mode = |bounds: &Punctuated<syn::TypeParamBound, Token![+]>| {
        bounds.iter().any(is_storage_mode_bound)
    };
    let inline = generics.type_params().find(|p| is_mode(&p.bounds));
    inline.map(|p| &p.ident).or_else(|| {
        generics
            .where_clause
            .iter()
            .flat_map(|w| &w.predicates)
            .find_map(|pred| match pred {
                WherePredicate::Type(pt) if is_mode(&pt.bounds) => match &pt.bounded_ty {
                    Type::Path(tp) => tp.path.get_ident(),
                    _ => None,
                },
                _ => None,
            })
    })
}

#[proc_macro_derive(Traversable, attributes(traversable))]
pub fn derive_traversable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive(&input) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn derive(input: &DeriveInput) -> Result<ProcMacro2TokenStream> {
    let (merge, client) = parse_struct_options(&input.attrs)?;
    let fields = parse_fields(input, merge)?;
    let client_name = client_name(input, &fields, merge, client)?;
    let mut output = gen_traversable(input, &fields, merge, client_name.as_deref());
    output.extend(gen_read_only_clone(input, &fields));
    Ok(output)
}

/// The name client types take for the struct's branch: the one it declares, else its identifier
/// without a `Vecs` suffix (`PoolVecs` is `Pool`); a plain `Vecs` names nothing. A struct that is
/// its field's node (a tuple struct, a lone flattened field) has no branch to name.
fn client_name(
    input: &DeriveInput,
    fields: &[FieldModel],
    merge: bool,
    client: Option<LitStr>,
) -> Result<Option<String>> {
    if delegates(fields, merge) {
        return match client {
            Some(client) => Err(Error::new_spanned(
                client,
                "`client` names a branch, and this struct is its field's node",
            )),
            None => Ok(None),
        };
    }
    if let Some(client) = client {
        return Ok(Some(client.value()));
    }
    let ident = input.ident.to_string();
    let name = ident.strip_suffix("Vecs").unwrap_or(&ident);
    Ok(is_type_name(name).then(|| name.to_owned()))
}

/// Whether the struct's node is its one field's: a tuple struct's, or a lone visible flattened
/// field's.
fn delegates(fields: &[FieldModel], merge: bool) -> bool {
    let visible: Vec<&FieldModel> = fields.iter().filter(|f| !f.skipped && !f.hidden).collect();
    matches!(fields, [field] if matches!(field.member, Member::Unnamed(_)))
        || (!merge && matches!(visible[..], [only] if only.flatten && only.rename.is_none()))
}

fn gen_traversable(
    input: &DeriveInput,
    fields: &[FieldModel],
    merge: bool,
    client: Option<&str>,
) -> ProcMacro2TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, _) = generics.split_for_impl();

    if let [field] = fields
        && let Member::Unnamed(_) = field.member
    {
        let where_clause = build_where_clause(generics, &[field.ty]);
        let collect_description = with_description_fragment(
            field,
            quote! {
                self.0.collect_series_descriptions(description_fragments, descriptions);
            },
        );
        return quote! {
            impl #impl_generics bitview_traversable::Traversable for #name #ty_generics #where_clause {
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
                    description_fragments: &mut Vec<bitview_traversable::Fragment>,
                    descriptions: &mut std::collections::BTreeMap<&'a str, Vec<bitview_traversable::Fragment>>,
                ) {
                    #collect_description
                }
            }
        };
    }

    let traversed: Vec<&FieldModel> = fields.iter().filter(|f| !f.skipped).collect();
    let visible: Vec<&FieldModel> = traversed.iter().copied().filter(|f| !f.hidden).collect();

    let traversed_types: Vec<&Type> = traversed.iter().map(|f| f.ty).collect();
    let where_clause = build_where_clause(generics, &traversed_types);

    let to_tree_node_body = generate_field_traversals(client, &visible, merge);
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
        impl #impl_generics bitview_traversable::Traversable for #name #ty_generics #where_clause {
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

fn generate_field_traversals(
    client: Option<&str>,
    visible: &[&FieldModel],
    merge: bool,
) -> ProcMacro2TokenStream {
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
    let field_operations = visible.iter().map(|field| {
        let member = &field.member;
        let Member::Named(field_name) = member else {
            unreachable!("only tuple structs have unnamed fields, and they delegate");
        };
        if field.flatten {
            // A flattened leaf keeps a key: its rename, else the field name.
            let leaf_key = field
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

        // With `wrap = "a/b"`, the node appears at `a/b/<rename-or-field-name>`.
        let key = field.rename.as_deref().unwrap_or(&field_name_str);
        let (outer_key, wrap_path) = match field.wrap.as_deref() {
            Some(wrap) => {
                let mut parts = wrap.split('/');
                let outer = parts.next().unwrap();
                let path = parts.chain(iter::once(key)).collect::<Vec<_>>();
                (outer, path)
            }
            None => (key, vec![]),
        };

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

    // The branch records its client name, for client type names.
    let client_name = match client {
        Some(name) => quote! { Some(#name) },
        None => quote! { None },
    };
    quote! {
        let mut collected = bitview_traversable::TreeBranch::default();
        #(#field_operations)*
        let mut node = #final_expr;
        if let bitview_traversable::TreeNode::Branch(branch) = &mut node {
            branch.client_name = #client_name;
        }
        node
    }
}

/// Collects each visible field's descriptions under its doc comment. Without visible fields
/// the trait's default (which visits no vec) applies.
fn generate_description_impl(visible: &[&FieldModel]) -> ProcMacro2TokenStream {
    if visible.is_empty() {
        return quote! {};
    }

    let fields = visible.iter().map(|field| {
        let member = &field.member;
        with_description_fragment(
            field,
            quote! {
                self.#member.collect_series_descriptions(description_fragments, descriptions);
            },
        )
    });

    quote! {
        fn collect_series_descriptions<'a>(
            &'a self,
            description_fragments: &mut Vec<bitview_traversable::Fragment>,
            descriptions: &mut std::collections::BTreeMap<&'a str, Vec<bitview_traversable::Fragment>>,
        ) {
            #(#fields)*
        }
    }
}

/// `ReadOnlyClone` for a derived type: a `StorageMode` param maps `Rw` to `Ro`. Otherwise container
/// params (unbounded, or used as a bare non-skipped field type) propagate it; without any, nothing
/// is generated.
fn gen_read_only_clone(input: &DeriveInput, fields: &[FieldModel]) -> ProcMacro2TokenStream {
    let generics = &input.generics;
    let name = &input.ident;

    if let Some(mode_param) = find_storage_mode_param(generics) {
        return gen_read_only_clone_storage_mode(name, generics, fields, mode_param);
    }

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
        if let Type::Path(type_path) = field.ty
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

fn gen_read_only_clone_field_value(
    field: &FieldModel,
    is_relevant: impl Fn(&Type) -> bool,
) -> ProcMacro2TokenStream {
    let member = &field.member;
    let ty = field.ty;
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

/// Tuple structs use named-field syntax: `Name { 0: .. }`.
fn gen_read_only_clone_body(
    name: &Ident,
    fields: &[FieldModel],
    is_relevant: impl Fn(&Type) -> bool,
) -> ProcMacro2TokenStream {
    let conversions = fields.iter().map(|field| {
        let member = &field.member;
        let value = gen_read_only_clone_field_value(field, &is_relevant);
        quote! { #member: #value }
    });
    quote! { #name { #(#conversions,)* } }
}

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
        .filter(|f| !is_write_only_type(f.ty))
        .filter_map(|f| -> Option<WherePredicate> {
            let ty = f.ty;
            if f.skipped {
                return type_projects_param(ty, &other_params).then(|| parse_quote! { #ty: Clone });
            }
            let generic = other_params.iter().any(|p| type_contains_ident(ty, p));
            if generic && type_contains_ident(ty, mode_param) {
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

    // The impl keeps every other param, its bounds and the struct's predicates, with the mode
    // param they name now `Rw`; `ReadOnly` instantiates the struct with `Ro`, so those naming
    // the mode param also hold with `Ro`.
    let rw = quote! { bitview_traversable::Rw };
    let ro = quote! { bitview_traversable::Ro };
    let mut mode_predicates: Vec<WherePredicate> = Vec::new();
    // The mode param's own inline bounds besides `StorageMode` hold for both modes (`?Sized`
    // relaxes a param and means nothing on a concrete type).
    let mode_bounds: Vec<_> = generics
        .type_params()
        .filter(|p| p.ident == *mode_param)
        .flat_map(|p| &p.bounds)
        .filter(|b| !is_storage_mode_bound(b) && !is_maybe_bound(b))
        .collect();
    if !mode_bounds.is_empty() {
        let bounds = quote! { #(#mode_bounds)+* };
        for mode in [&rw, &ro] {
            let bounds = substitute_mode(bounds.clone(), mode_param, mode);
            mode_predicates.push(parse_quote! { #mode: #bounds });
        }
    }
    let mut impl_generics = generics.clone();
    impl_generics.params = generics
        .params
        .iter()
        .filter(|p| !matches!(p, GenericParam::Type(tp) if tp.ident == *mode_param))
        .cloned()
        .collect();
    for param in impl_generics.type_params_mut() {
        let ident = &param.ident;
        let bounds = &param.bounds;
        let bounds = quote! { #bounds };
        if tokens_contain_ident(&bounds, mode_param) {
            let bounds = substitute_mode(bounds.clone(), mode_param, &ro);
            mode_predicates.push(parse_quote! { #ident: #bounds });
        }
        let bounds = substitute_mode(bounds, mode_param, &rw);
        param.bounds = parse_quote! { #bounds };
    }
    if let Some(where_clause) = &mut impl_generics.where_clause {
        // `?Sized` on the mode param relaxes a param and means nothing once it is concrete.
        where_clause.predicates = std::mem::take(&mut where_clause.predicates)
            .into_iter()
            .filter_map(|mut predicate| {
                if let WherePredicate::Type(pt) = &mut predicate
                    && matches!(&pt.bounded_ty, Type::Path(tp) if tp.path.is_ident(mode_param))
                {
                    pt.bounds = std::mem::take(&mut pt.bounds)
                        .into_iter()
                        .filter(|b| !is_maybe_bound(b))
                        .collect();
                    if pt.bounds.is_empty() {
                        return None;
                    }
                }
                Some(predicate)
            })
            .collect();
        for predicate in &mut where_clause.predicates {
            let tokens = quote! { #predicate };
            if tokens_contain_ident(&tokens, mode_param) {
                let tokens = substitute_mode(tokens.clone(), mode_param, &ro);
                mode_predicates.push(parse_quote! { #tokens });
            }
            let tokens = substitute_mode(tokens, mode_param, &rw);
            *predicate = parse_quote! { #tokens };
        }
    }
    impl_generics
        .make_where_clause()
        .predicates
        .extend(mode_predicates.into_iter().chain(field_bounds));
    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();

    let body = gen_read_only_clone_body(name, fields, |ty| type_contains_ident(ty, mode_param));

    quote! {
        impl #impl_generics bitview_traversable::ReadOnlyClone for #name<#(#ty_args_rw),*> #where_clause {
            type ReadOnly = #name<#(#ty_args_ro),*>;

            fn read_only_clone(&self) -> Self::ReadOnly {
                #body
            }
        }
    }
}

fn gen_read_only_clone_generics(
    name: &Ident,
    generics: &Generics,
    fields: &[FieldModel],
    container_params: &[&Ident],
) -> ProcMacro2TokenStream {
    let has_container_field = fields.iter().any(|f| {
        !f.skipped
            && container_params
                .iter()
                .any(|tp| type_contains_ident(f.ty, tp))
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

    let body = gen_read_only_clone_body(name, fields, |ty| {
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
