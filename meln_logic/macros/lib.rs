use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed};

#[proc_macro_derive(PropertiesExt, attributes(props))]
pub fn derive_helper(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);
    let struct_ident = ident.clone();
    println!("My Derive beggin: {}", quote! {#ident});
    if let syn::Data::Struct(s) = data {
        if let syn::Fields::Named(FieldsNamed { named, .. }) = s.fields {
            let mut props_read = Vec::new();
            let mut props = None;
            for n in named {
                let name = n.ident;
                
                if n.attrs.len()>0 {
                    let attr = n.attrs[0].clone();
                    let ty = n.ty.clone();
                    let p = quote! {[#attr] #name: #ty};
                    println!("Field Attr {}", p);
                }
                
                let ty = n.ty.clone();
                match format!("{}", quote!{#ty}).as_str() {
                "PropertyRead < f32 >" => {
                    props_read.push(name.clone());
                    let ty = n.ty.clone();
                    let p = quote! {#name: #ty};
                    println!("Field Type {}", p);
                }, "Properties" => {
                    props = Some(name.clone());
                }, _ => {}
                }
            }
            
            if let Some(props) = props {
                return quote!{
                    impl PropertiesExt for #struct_ident {
//                         fn init_props() -> Self {
//                             let prop_str = [#(stringify!(#props_read)),*];
//                             
//                             Self {
//                                 #(#props_read): Default::default(), *
//                                 #props: Properties::new(&prop_str),
//                             }
//                         }
                        fn get_props(&self) -> String {
                            let prop_str = [#(stringify!(#props_read)),*];
                            prop_str.join(", ").into()
                        }
                    }
                }.into();
            }
        }
    }
    TokenStream::new()
}
