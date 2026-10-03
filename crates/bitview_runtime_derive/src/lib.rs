use proc_macro::TokenStream;
use proc_macro2::TokenStream as ProcMacro2TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Field, Fields, GenericArgument, Path, PathArguments, Result, Type,
    WherePredicate, parse_macro_input, parse_quote,
};

enum FieldKind {
    Plugin,
    Flatten,
    Skip,
}

/// The field's role, plus the capability traits (`has = Trait<..>`) whose single
/// accessor method, named after the field, returns it.
fn field_attributes(field: &Field) -> Result<(FieldKind, Vec<Path>)> {
    let mut kind = FieldKind::Plugin;
    let mut capabilities = Vec::new();

    for attribute in &field.attrs {
        if !attribute.path().is_ident("plugin_set") {
            continue;
        }
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("flatten") {
                kind = FieldKind::Flatten;
            } else if meta.path.is_ident("skip") {
                kind = FieldKind::Skip;
            } else if meta.path.is_ident("has") {
                capabilities.push(meta.value()?.parse()?);
            } else {
                return Err(meta.error("expected `flatten`, `skip` or `has = Trait<..>`"));
            }
            Ok(())
        })?;
    }

    Ok((kind, capabilities))
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
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn derive_plugin_set_inner(input: DeriveInput) -> Result<ProcMacro2TokenStream> {
    let name = input.ident;
    let Data::Struct(data) = input.data else {
        return Err(Error::new_spanned(
            name,
            "PluginSet can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = data.fields else {
        return Err(Error::new_spanned(name, "PluginSet requires named fields"));
    };

    let mut generics = input.generics;
    let (own_impl_generics, own_ty_generics, own_where_clause) = generics.split_for_impl();
    let mut capability_impls = Vec::new();
    let mut visits = Vec::new();
    let mut predicates = Vec::<WherePredicate>::new();

    for field in fields.named {
        let (kind, capabilities) = field_attributes(&field)?;
        let ident = field.ident.expect("named fields have identifiers");
        let ty = field.ty;
        let inner = boxed_inner(&ty);
        let reference = if inner.is_some() {
            quote! { self.#ident.as_ref() }
        } else {
            quote! { &self.#ident }
        };
        let field_ty = inner.unwrap_or(&ty);

        for capability in capabilities {
            capability_impls.push(quote! {
                impl #own_impl_generics #capability for #name #own_ty_generics #own_where_clause {
                    fn #ident(&self) -> &#field_ty {
                        #reference
                    }
                }
            });
        }

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

        #(#capability_impls)*
    })
}
