use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericArgument, Meta, PathArguments, Type, parse_macro_input,
    parse_quote,
};

enum FieldKind {
    Plugin,
    Flatten,
    Skip,
}

fn field_kind(field: &syn::Field) -> syn::Result<FieldKind> {
    let mut kind = FieldKind::Plugin;

    for attribute in &field.attrs {
        if !attribute.path().is_ident("plugin_set") {
            continue;
        }

        let metadata = attribute.parse_args_with(
            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
        )?;

        for meta in metadata {
            match meta {
                Meta::Path(path) if path.is_ident("flatten") => kind = FieldKind::Flatten,
                Meta::Path(path) if path.is_ident("skip") => kind = FieldKind::Skip,
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "expected `flatten` or `skip`",
                    ));
                }
            }
        }
    }

    Ok(kind)
}

fn boxed_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Box" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    if arguments.args.len() != 1 {
        return None;
    }
    match arguments.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

#[proc_macro_derive(PluginSet, attributes(plugin_set))]
pub fn derive_plugin_set(input: TokenStream) -> TokenStream {
    derive_plugin_set_inner(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_plugin_set_inner(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "PluginSet can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "PluginSet requires named fields",
        ));
    };

    let mut generics = input.generics;
    let mut visits = Vec::new();
    let mut predicates = Vec::<syn::WherePredicate>::new();

    for field in fields.named {
        let kind = field_kind(&field)?;
        let ident = field.ident.expect("named fields have identifiers");
        let ty = field.ty;
        let inner = boxed_inner(&ty);
        let reference = if inner.is_some() {
            quote! { self.#ident.as_ref() }
        } else {
            quote! { &self.#ident }
        };
        let field_ty = inner.unwrap_or(&ty);

        match kind {
            FieldKind::Plugin => {
                visits.push(quote! { visit(#reference); });
                predicates.push(parse_quote!(#field_ty: bitview_plugin::Plugin));
            }
            FieldKind::Flatten => {
                visits.push(quote! {
                    bitview_runtime::PluginSet::for_each_plugin(#reference, visit);
                });
                predicates.push(parse_quote!(#field_ty: bitview_runtime::PluginSet));
            }
            FieldKind::Skip => {}
        }
    }

    let where_clause = generics.make_where_clause();
    where_clause
        .predicates
        .push(parse_quote!(Self: Send + Sync));
    where_clause.predicates.extend(predicates);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics bitview_runtime::PluginSet for #name #ty_generics #where_clause {
            fn for_each_plugin<'a>(
                &'a self,
                visit: &mut dyn FnMut(&'a dyn bitview_plugin::Plugin),
            ) {
                #(#visits)*
            }
        }
    })
}
