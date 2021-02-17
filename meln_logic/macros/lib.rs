use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed};

#[proc_macro_derive(PropertiesExt, attributes(props))]
pub fn derive_helper(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);
    println!("My Derive beggin: {}", quote! {#ident});
    if let syn::Data::Struct(s) = data {
        if let syn::Fields::Named(FieldsNamed { named, .. }) = s.fields {
            for n in named {
                let name = n.ident;
                
                if n.attrs.len()>0 {
                    let attr = n.attrs[0].clone();
                    let ty = n.ty.clone();
                    let p = quote! {[#attr] #name: #ty};
                    println!("Field Attr {}", p);
                }
                
                let ty = n.ty.clone();
                if format!("{}", quote!{#ty}) == "PropertyRead < f32 >" {
                    let ty = n.ty.clone();
                    let p = quote! {#name: #ty};
                    println!("Field Type {}", p);
                }
            }
        }
    }
    TokenStream::new()
}
