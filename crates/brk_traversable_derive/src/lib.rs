use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

// ===========================================================================
// Struct & field attribute parsing
// ===========================================================================

#[derive(Default)]
struct StructAttr {
    merge: bool,
    transparent: bool,
    hidden: bool,
    wrap: Option<String>,
}

fn get_struct_attr(attrs: &[syn::Attribute]) -> StructAttr {
    let mut result = StructAttr::default();
    for attr in attrs {
        if !attr.path().is_ident("traversable") {
            continue;
        }

        if let Ok(ident) = attr.parse_args::<syn::Ident>() {
            match ident.to_string().as_str() {
                "merge" => result.merge = true,
                "transparent" => result.transparent = true,
                "hidden" => result.hidden = true,
                _ => {}
            }
            continue;
        }

        if let Ok(meta) = attr.parse_args::<syn::MetaNameValue>()
            && meta.path.is_ident("wrap")
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
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
    name: &'a syn::Ident,
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
fn get_field_attr(field: &syn::Field) -> Option<ParsedFieldAttr> {
    let mut attr_type = FieldAttr::Normal;
    let mut rename = None;
    let mut wrap = None;
    let mut hidden = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("traversable") {
            continue;
        }

        let Ok(metas) = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        ) else {
            continue;
        };

        for meta in metas {
            match meta {
                syn::Meta::Path(path) if path.is_ident("skip") => return None,
                syn::Meta::Path(path) if path.is_ident("flatten") => {
                    attr_type = FieldAttr::Flatten;
                }
                syn::Meta::Path(path) if path.is_ident("hidden") => hidden = true,
                syn::Meta::NameValue(meta) => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
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

fn get_doc_comment(field: &syn::Field) -> Option<String> {
    let lines = field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| match &attr.meta {
            syn::Meta::NameValue(meta) => match &meta.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(value),
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
    collect: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let Some(description) = description else {
        return collect;
    };

    quote! {{
        description_fragments.push(#description);
        #collect
        debug_assert_eq!(description_fragments.pop(), Some(#description));
    }}
}

fn is_field_skipped(field: &syn::Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.path().is_ident("traversable")
            && attr.parse_args::<syn::Ident>().is_ok_and(|id| id == "skip")
    })
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
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        Some(inner)
    } else {
        None
    }
}

/// Check if a type AST references the given identifier anywhere.
fn type_contains_ident(ty: &Type, ident: &syn::Ident) -> bool {
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
                    syn::PathArguments::AngleBracketed(args) => args.args.iter().any(|arg| {
                        matches!(arg, syn::GenericArgument::Type(inner) if type_contains_ident(inner, ident))
                    }),
                    syn::PathArguments::Parenthesized(args) => {
                        args.inputs.iter().any(|inner| type_contains_ident(&inner.ty, ident))
                            || matches!(&args.output, syn::ReturnType::Type(_, inner) if type_contains_ident(inner, ident))
                    }
                    syn::PathArguments::None => false,
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

/// Find the generic type parameter bounded by `StorageMode`, if any.
fn find_storage_mode_param(generics: &syn::Generics) -> Option<&syn::Ident> {
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

fn gen_traversable(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, _) = generics.split_for_impl();

    let struct_attr = get_struct_attr(&input.attrs);

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(
            &input.ident,
            "Traversable can only be derived for structs",
        )
        .to_compile_error();
    };

    // Single-field tuple struct: delegate (automatic transparent).
    if let Fields::Unnamed(fields) = &data.fields
        && fields.unnamed.len() == 1
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
            quote! { brk_traversable::TreeNode::wrap(#wrap_key, self.0.to_tree_node()) }
        } else {
            quote! { self.0.to_tree_node() }
        };
        return quote! {
            impl #impl_generics Traversable for #name #ty_generics #where_clause {
                fn to_tree_node(&self) -> brk_traversable::TreeNode {
                    #to_tree_node_body
                }

                fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn vecdb::AnyExportableVec> {
                    self.0.iter_any_exportable()
                }

                fn iter_any_visible(&self) -> impl Iterator<Item = &dyn vecdb::AnyExportableVec> {
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
                fn to_tree_node(&self) -> brk_traversable::TreeNode {
                    brk_traversable::TreeNode::Branch(brk_traversable::IndexMap::new())
                }

                fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn vecdb::AnyExportableVec> {
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

    // Transparent delegation: forward everything to the first field.
    if struct_attr.transparent {
        let first_field = named_fields
            .named
            .first()
            .expect("transparent requires at least one field");
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
                fn to_tree_node(&self) -> brk_traversable::TreeNode {
                    self.#field_name.to_tree_node()
                }

                fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn vecdb::AnyExportableVec> {
                    self.#field_name.iter_any_exportable()
                }

                fn iter_any_visible(&self) -> impl Iterator<Item = &dyn vecdb::AnyExportableVec> {
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
        quote! { brk_traversable::TreeNode::Branch(brk_traversable::IndexMap::new()) }
    } else {
        field_traversals
    };

    quote! {
        impl #impl_generics Traversable for #name #ty_generics #where_clause {
            fn to_tree_node(&self) -> brk_traversable::TreeNode {
                #to_tree_node_body
            }

            #iterator_impl
            #description_impl
        }
    }
}

fn analyze_fields<'a>(
    fields: &'a syn::FieldsNamed,
    generic_params: &[&'a syn::Ident],
) -> (Vec<FieldInfo<'a>>, Vec<&'a syn::Ident>, Vec<&'a syn::Type>) {
    let mut field_infos = Vec::new();
    let mut generics_set = std::collections::BTreeSet::new();
    let mut field_traversable_types = Vec::new();

    for field in &fields.named {
        let Some(parsed) = get_field_attr(field) else {
            continue;
        };

        // Hidden fields stay out of the public tree but must remain in
        // exportable traversal for storage retention.
        if !parsed.hidden && !matches!(field.vis, syn::Visibility::Public(_)) {
            continue;
        }

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
    generics: &syn::Generics,
    generics_needing_traversable: &[&syn::Ident],
    extra_traversable_types: &[&syn::Type],
) -> proc_macro2::TokenStream {
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
            #(#generics_needing_traversable: brk_traversable::Traversable,)*
            #(#extra_traversable_types: brk_traversable::Traversable,)*
            #(#generic_params: Send + Sync,)*
            #original_predicates
    }
}

fn generate_field_traversals(infos: &[FieldInfo], merge: bool) -> proc_macro2::TokenStream {
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
                        let s = s.strip_prefix("r#").unwrap_or(&s).to_string();
                        s.strip_prefix('_').map(String::from).unwrap_or(s)
                    };

                    // Determine the tree key and optional wrapping path.
                    // wrap = "a/b" means: outer_key = "a", wrap the node under "b" then under the rename/field name.
                    // wrap = "a" means: outer_key = "a", wrap under rename or field name.
                    // No wrap: outer_key = rename or field name, no wrapping.
                    let (outer_key, wrap_path): (String, Vec<&str>) =
                        match (info.wrap.as_deref(), info.rename.as_deref()) {
                            (Some(wrap), Some(rename)) => {
                                let parts: Vec<&str> = wrap.split('/').collect();
                                let outer = parts[0].to_string();
                                let mut path: Vec<&str> = parts[1..].to_vec();
                                path.push(rename);
                                (outer, path)
                            }
                            (Some(wrap), None) => {
                                let parts: Vec<&str> = wrap.split('/').collect();
                                let outer = parts[0].to_string();
                                let mut path: Vec<&str> = parts[1..].to_vec();
                                path.push(&field_name_str);
                                (outer, path)
                            }
                            (None, Some(rename)) => (rename.to_string(), vec![]),
                            (None, None) => (field_name_str.clone(), vec![]),
                        };

                    // Build nested wrapping: wrap(path[last], wrap(path[last-1], ... node))
                    let build_wrapped = |base: proc_macro2::TokenStream| -> proc_macro2::TokenStream {
                        wrap_path.iter().rev().fold(base, |inner, key| {
                            quote! { brk_traversable::TreeNode::wrap(#key, #inner) }
                        })
                    };

                    if info.is_option {
                        let node_expr = build_wrapped(quote! { nested.to_tree_node() });
                        quote! {
                            if let Some(entry) = self.#field_name.as_ref().map(|nested| (String::from(#outer_key), #node_expr)) {
                                brk_traversable::TreeNode::merge_node(&mut collected, entry.0, entry.1)
                                    .expect("Conflicting values for same key");
                            }
                        }
                    } else {
                        let node_expr_self = build_wrapped(quote! { self.#field_name.to_tree_node() });
                        quote! {
                            brk_traversable::TreeNode::merge_node(&mut collected, String::from(#outer_key), #node_expr_self)
                                .expect("Conflicting values for same key");
                        }
                    }
                }
                FieldAttr::Flatten => {
                    let field_name = info.name;
                    let merge_branch = quote! {
                        brk_traversable::TreeNode::Branch(map) => {
                            for (key, node) in map {
                                brk_traversable::TreeNode::merge_node(&mut collected, key, node)
                                    .expect("Conflicting values for same key during flatten");
                            }
                        }
                        leaf @ brk_traversable::TreeNode::Leaf(_) => {
                            brk_traversable::TreeNode::merge_node(&mut collected, String::from(stringify!(#field_name)), leaf)
                                .expect("Conflicting values for same key during flatten");
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
        quote! { brk_traversable::TreeNode::Branch(collected).merge_branches().unwrap() }
    } else {
        quote! { brk_traversable::TreeNode::Branch(collected) }
    };

    let init_collected = quote! {
        let mut collected: brk_traversable::IndexMap<String, brk_traversable::TreeNode> =
            brk_traversable::IndexMap::new();
    };

    quote! {
        #init_collected
        #(#field_operations)*
        #final_expr
    }
}

fn generate_iter_body(
    fields: &[&syn::Ident],
    option_fields: &[&syn::Ident],
    method: &str,
) -> proc_macro2::TokenStream {
    let method_ident = syn::Ident::new(method, proc_macro2::Span::call_site());

    if fields.is_empty() && option_fields.is_empty() {
        return quote! { std::iter::empty() };
    }

    let (init_part, chain_part) = if let Some((&first, rest)) = fields.split_first() {
        (
            quote! {
                let mut iter: Box<dyn Iterator<Item = &dyn vecdb::AnyExportableVec>> =
                    Box::new(self.#first.#method_ident());
            },
            quote! {
                #(iter = Box::new(iter.chain(self.#rest.#method_ident()));)*
            },
        )
    } else {
        (
            quote! {
                let mut iter: Box<dyn Iterator<Item = &dyn vecdb::AnyExportableVec>> =
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

fn generate_iterator_impl(infos: &[FieldInfo], struct_hidden: bool) -> proc_macro2::TokenStream {
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
            fn iter_any_visible(&self) -> impl Iterator<Item = &dyn vecdb::AnyExportableVec> {
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
            fn iter_any_visible(&self) -> impl Iterator<Item = &dyn vecdb::AnyExportableVec> {
                #visible_body
            }
        }
    };

    quote! {
        fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn vecdb::AnyExportableVec> {
            #exportable_body
        }
        #visible_impl
    }
}

fn generate_description_impl(infos: &[FieldInfo], struct_hidden: bool) -> proc_macro2::TokenStream {
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
fn gen_read_only_clone(input: &DeriveInput) -> proc_macro2::TokenStream {
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
    let type_params: Vec<&syn::TypeParam> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(tp) => Some(tp),
            _ => None,
        })
        .collect();

    if type_params.is_empty() {
        return quote! {};
    }

    let is_bounded = |tp: &syn::TypeParam| -> bool {
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

    let container_params: Vec<&syn::Ident> = type_params
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
fn find_bare_field_params<'a>(
    data: &syn::DataStruct,
    type_params: &[&'a syn::TypeParam],
) -> Vec<&'a syn::Ident> {
    let fields: &syn::punctuated::Punctuated<syn::Field, _> = match &data.fields {
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
/// - Skipped + Option → `None`
/// - Skipped + non-Option → `Default::default()`
/// - Contains relevant param + Box → `Box::new(read_only_clone(&*self.field))`
/// - Contains relevant param → `read_only_clone(&self.field)`
/// - Otherwise → `self.field.clone()`
fn gen_roc_field_value(
    field: &syn::Field,
    self_access: proc_macro2::TokenStream,
    is_relevant: impl Fn(&Type) -> bool,
) -> proc_macro2::TokenStream {
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
            quote! { Box::new(vecdb::ReadOnlyClone::read_only_clone(&*#self_access)) }
        } else {
            quote! { vecdb::ReadOnlyClone::read_only_clone(&#self_access) }
        }
    } else {
        quote! { #self_access.clone() }
    }
}

/// Generate the struct body for a ReadOnlyClone impl.
fn gen_roc_body(
    name: &syn::Ident,
    data: &syn::DataStruct,
    is_relevant: impl Fn(&Type) -> bool,
) -> proc_macro2::TokenStream {
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
                    let idx = syn::Index::from(i);
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
    generics: &syn::Generics,
    map_type: impl Fn(&syn::TypeParam) -> proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    generics
        .params
        .iter()
        .map(|p| match p {
            syn::GenericParam::Type(tp) => map_type(tp),
            syn::GenericParam::Lifetime(lt) => {
                let lt = &lt.lifetime;
                quote! { #lt }
            }
            syn::GenericParam::Const(c) => {
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
    name: &syn::Ident,
    generics: &syn::Generics,
    data: &syn::DataStruct,
    mode_param: &syn::Ident,
) -> proc_macro2::TokenStream {
    let impl_params: Vec<proc_macro2::TokenStream> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(tp) if tp.ident == *mode_param => None,
            syn::GenericParam::Type(tp) => {
                let ident = &tp.ident;
                let bounds = &tp.bounds;
                if bounds.is_empty() {
                    Some(quote! { #ident })
                } else {
                    Some(quote! { #ident: #bounds })
                }
            }
            syn::GenericParam::Lifetime(lt) => Some(quote! { #lt }),
            syn::GenericParam::Const(c) => {
                let ident = &c.ident;
                let ty = &c.ty;
                Some(quote! { const #ident: #ty })
            }
        })
        .collect();

    let make_ty_args = |replacement: proc_macro2::TokenStream| {
        collect_ty_args(generics, |tp| {
            if tp.ident == *mode_param {
                replacement.clone()
            } else {
                let id = &tp.ident;
                quote! { #id }
            }
        })
    };

    let ty_args_rw = make_ty_args(quote! { vecdb::Rw });
    let ty_args_ro = make_ty_args(quote! { vecdb::Ro });
    let where_clause = &generics.where_clause;

    let body = gen_roc_body(name, data, |ty| type_contains_ident(ty, mode_param));

    let impl_generics = if impl_params.is_empty() {
        quote! {}
    } else {
        quote! { <#(#impl_params),*> }
    };

    quote! {
        impl #impl_generics vecdb::ReadOnlyClone for #name<#(#ty_args_rw),*> #where_clause {
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
    name: &syn::Ident,
    generics: &syn::Generics,
    data: &syn::DataStruct,
    type_params: &[&syn::TypeParam],
    container_params: &[&syn::Ident],
) -> proc_macro2::TokenStream {
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

    let is_container = |ident: &syn::Ident| container_params.contains(&ident);

    // Impl params: containers get ReadOnlyClone (+ original bounds), others keep their bounds.
    let impl_params: Vec<proc_macro2::TokenStream> = generics
        .params
        .iter()
        .map(|p| match p {
            syn::GenericParam::Type(tp) => {
                let ident = &tp.ident;
                let bounds = &tp.bounds;
                if is_container(ident) {
                    if bounds.is_empty() {
                        quote! { #ident: vecdb::ReadOnlyClone }
                    } else {
                        quote! { #ident: #bounds + vecdb::ReadOnlyClone }
                    }
                } else if bounds.is_empty() {
                    quote! { #ident }
                } else {
                    quote! { #ident: #bounds }
                }
            }
            syn::GenericParam::Lifetime(lt) => quote! { #lt },
            syn::GenericParam::Const(c) => {
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
            quote! { <#id as vecdb::ReadOnlyClone>::ReadOnly }
        } else {
            quote! { #id }
        }
    });

    // Where clause: propagate bounds from bounded container params to their ReadOnly.
    let mut extra_where: Vec<proc_macro2::TokenStream> = Vec::new();

    for tp in type_params {
        if is_container(&tp.ident) && !tp.bounds.is_empty() {
            let ident = &tp.ident;
            let bounds = &tp.bounds;
            extra_where.push(quote! {
                <#ident as vecdb::ReadOnlyClone>::ReadOnly: #bounds
            });
        }
    }

    if let Some(wc) = &generics.where_clause {
        for pred in &wc.predicates {
            if let syn::WherePredicate::Type(pt) = pred
                && let Type::Path(tp) = &pt.bounded_ty
                && let Some(seg) = tp.path.segments.first()
                && container_params.iter().any(|cp| **cp == seg.ident)
            {
                let ident = &seg.ident;
                let bounds = &pt.bounds;
                extra_where.push(quote! {
                    <#ident as vecdb::ReadOnlyClone>::ReadOnly: #bounds
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
        impl<#(#impl_params),*> vecdb::ReadOnlyClone for #name<#(#self_ty_args),*> #combined_where {
            type ReadOnly = #name<#(#ro_ty_args),*>;

            fn read_only_clone(&self) -> Self::ReadOnly {
                #body
            }
        }
    }
}
