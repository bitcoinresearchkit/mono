use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, parse_macro_input};

/// Derives the `Bytes` trait for single-field tuple structs.
///
/// This macro enables custom wrapper types to work with `BytesVec`, `LZ4Vec`, `ZstdVec`,
/// and other vecdb vector types that require the `Bytes` trait.
///
/// # Requirements
///
/// - Must be a tuple struct with exactly one field
/// - The inner type must implement `Bytes`
/// - Supports generic type parameters
///
/// # Generated Implementation
///
/// The derive generates a `Bytes` implementation that delegates to the inner type:
///
/// ```rust,ignore
/// impl Bytes for Wrapper<T> where T: Bytes {
///     type Array = <T as Bytes>::Array;
///
///     fn to_bytes(&self) -> Self::Array {
///         self.0.to_bytes()
///     }
///     fn from_bytes(bytes: &[u8]) -> Result<Self> {
///         Ok(Self(<T>::from_bytes(bytes)?))
///     }
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use vecdb::{Bytes, BytesVec};
///
/// #[derive(Bytes)]
/// struct UserId(u64);
///
/// #[derive(Bytes)]
/// struct Timestamp<T>(T); // Generic types supported
/// ```
#[proc_macro_derive(Bytes)]
pub fn derive_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let inner_type = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(fields),
            ..
        }) if fields.unnamed.len() == 1 => &fields.unnamed[0].ty,
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "Bytes can only be derived for single-field tuple structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Check if we have generic parameters
    let has_generics = !generics.params.is_empty();

    let expanded = if has_generics {
        let where_clause = if where_clause.is_some() {
            quote! { #where_clause #inner_type: ::vecdb::Bytes, }
        } else {
            quote! { where #inner_type: ::vecdb::Bytes, }
        };

        quote! {
            impl #impl_generics ::vecdb::Bytes for #struct_name #ty_generics #where_clause {
                type Array = <#inner_type as ::vecdb::Bytes>::Array;
                const IS_NATIVE_LAYOUT: bool =
                    <#inner_type as ::vecdb::Bytes>::IS_NATIVE_LAYOUT
                    && ::core::mem::size_of::<Self>() == ::core::mem::size_of::<#inner_type>()
                    && ::core::mem::align_of::<Self>() == ::core::mem::align_of::<#inner_type>();

                fn to_bytes(&self) -> Self::Array {
                    self.0.to_bytes()
                }

                fn from_bytes(bytes: &[u8]) -> ::vecdb::Result<Self> {
                    Ok(Self(<#inner_type>::from_bytes(bytes)?))
                }
            }
        }
    } else {
        quote! {
            impl ::vecdb::Bytes for #struct_name {
                type Array = <#inner_type as ::vecdb::Bytes>::Array;
                const IS_NATIVE_LAYOUT: bool =
                    <#inner_type as ::vecdb::Bytes>::IS_NATIVE_LAYOUT
                    && ::core::mem::size_of::<Self>() == ::core::mem::size_of::<#inner_type>()
                    && ::core::mem::align_of::<Self>() == ::core::mem::align_of::<#inner_type>();

                fn to_bytes(&self) -> Self::Array {
                    self.0.to_bytes()
                }

                fn from_bytes(bytes: &[u8]) -> ::vecdb::Result<Self> {
                    Ok(Self(<#inner_type>::from_bytes(bytes)?))
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derives the `Pco` trait for single-field tuple structs containing numeric types.
///
/// This macro enables custom wrapper types to work with `PcoVec` for compressed storage
/// of numeric data using Pcodec compression.
///
/// # Requirements
///
/// - Must be a tuple struct with exactly one field
/// - The inner type must implement `Pco` (numeric types: u16-u64, i16-i64, f32, f64)
/// - Supports generic type parameters
///
/// # Generated Implementation
///
/// The derive generates two trait implementations:
///
/// 1. `Bytes` - For serialization (same as `#[derive(Bytes)]`)
/// 2. `Pco` - Specifies the numeric type for compression
/// ```rust,ignore
/// impl Pco for Wrapper<T> where T: Pco + Bytes {
///     type NumberType = <T as Pco>::NumberType;
///     // Conversion delegates to the wrapped value.
/// }
///
/// impl Bytes for Wrapper<T> where T: Pco + Bytes {
///     // ... same as Bytes derive
/// }
/// ```
///
/// The `NumberType` is automatically propagated from the inner type, ensuring the
/// wrapper has the same compression characteristics.
///
/// # Example
///
/// ```rust,ignore
/// use vecdb::{Pco, PcoVec};
///
/// #[derive(Pco)]
/// struct Price(f64);
///
/// #[derive(Pco)]
/// struct NumericWrapper<T>(T); // Generic types supported
///
/// // Nested generics work too
/// #[derive(Pco)]
/// struct Container<T>(NumericWrapper<T>);
/// ```
#[proc_macro_derive(Pco)]
pub fn derive_pco(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let inner_type = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(fields),
            ..
        }) if fields.unnamed.len() == 1 => &fields.unnamed[0].ty,
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "Pco can only be derived for single-field tuple structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Check if we have generic parameters
    let has_generics = !generics.params.is_empty();

    let expanded = if has_generics {
        // For generic types, we need both Pco and Bytes bounds because:
        // - Pco trait requires the NumberType
        // - We call to_bytes/from_bytes methods which require Bytes
        let where_clause = if where_clause.is_some() {
            quote! { #where_clause #inner_type: ::vecdb::Pco + ::vecdb::Bytes, }
        } else {
            quote! { where #inner_type: ::vecdb::Pco + ::vecdb::Bytes, }
        };

        quote! {
            impl #impl_generics ::vecdb::Bytes for #struct_name #ty_generics #where_clause {
                type Array = <#inner_type as ::vecdb::Bytes>::Array;
                const IS_NATIVE_LAYOUT: bool =
                    <#inner_type as ::vecdb::Bytes>::IS_NATIVE_LAYOUT
                    && ::core::mem::size_of::<Self>() == ::core::mem::size_of::<#inner_type>()
                    && ::core::mem::align_of::<Self>() == ::core::mem::align_of::<#inner_type>();

                fn to_bytes(&self) -> Self::Array {
                    self.0.to_bytes()
                }

                fn from_bytes(bytes: &[u8]) -> ::vecdb::Result<Self> {
                    Ok(Self(<#inner_type>::from_bytes(bytes)?))
                }
            }

            // SAFETY: When the inner value is transparent, the generated Bytes
            // layout checks guarantee that this one-field wrapper is too.
            unsafe impl #impl_generics ::vecdb::Pco for #struct_name #ty_generics #where_clause {
                type NumberType = <#inner_type as ::vecdb::Pco>::NumberType;
                const IS_TRANSPARENT: bool =
                    <#inner_type as ::vecdb::Pco>::IS_TRANSPARENT
                    && <Self as ::vecdb::Bytes>::IS_NATIVE_LAYOUT;

                #[inline(always)]
                fn to_number(self) -> Self::NumberType {
                    self.0.to_number()
                }

                #[inline(always)]
                fn from_number(value: Self::NumberType) -> ::vecdb::Result<Self> {
                    Ok(Self(<#inner_type as ::vecdb::Pco>::from_number(value)?))
                }
            }
        }
    } else {
        quote! {
            impl ::vecdb::Bytes for #struct_name {
                type Array = <#inner_type as ::vecdb::Bytes>::Array;
                const IS_NATIVE_LAYOUT: bool =
                    <#inner_type as ::vecdb::Bytes>::IS_NATIVE_LAYOUT
                    && ::core::mem::size_of::<Self>() == ::core::mem::size_of::<#inner_type>()
                    && ::core::mem::align_of::<Self>() == ::core::mem::align_of::<#inner_type>();

                fn to_bytes(&self) -> Self::Array {
                    self.0.to_bytes()
                }

                fn from_bytes(bytes: &[u8]) -> ::vecdb::Result<Self> {
                    Ok(Self(<#inner_type>::from_bytes(bytes)?))
                }
            }

            // SAFETY: When the inner value is transparent, the generated Bytes
            // layout checks guarantee that this one-field wrapper is too.
            unsafe impl ::vecdb::Pco for #struct_name {
                type NumberType = <#inner_type as ::vecdb::Pco>::NumberType;
                const IS_TRANSPARENT: bool =
                    <#inner_type as ::vecdb::Pco>::IS_TRANSPARENT
                    && <Self as ::vecdb::Bytes>::IS_NATIVE_LAYOUT;

                #[inline(always)]
                fn to_number(self) -> Self::NumberType {
                    self.0.to_number()
                }

                #[inline(always)]
                fn from_number(value: Self::NumberType) -> ::vecdb::Result<Self> {
                    Ok(Self(<#inner_type as ::vecdb::Pco>::from_number(value)?))
                }
            }
        }
    };

    TokenStream::from(expanded)
}
