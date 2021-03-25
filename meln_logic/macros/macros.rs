use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed};

#[proc_macro_derive(PropertiesExt, attributes(props))]
pub fn derive_helper(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);
    let struct_ident = ident.clone();
    print!("My Derive Properties: {}: ", quote! {#ident});
    if let syn::Data::Struct(s) = data {
        if let syn::Fields::Named(FieldsNamed { named, .. }) = s.fields {
            let mut props_read = Vec::new();
            let mut props_read_def = Vec::new();
            let mut props = None;
            for n in named {
                let name = n.ident;
                
                if n.attrs.len()>0 {
                    let attr = n.attrs[0].clone();
                    let ty = n.ty.clone();
                    let p = quote! {[#attr] #name: #ty};
//                     println!("Field Attr {}", p);
                }
                
                let ty = n.ty.clone();
                match format!("{}", quote!{#ty}).as_str() {
                "PropertyRead < f32 >" => {
                    props_read.push(name.clone());
                    print!("{}, ", quote!(#name));
//                     let ty = n.ty.clone();
//                     let p = quote! {#name: #ty};
//                     println!("Field Type {}", p);

                }, s if s.starts_with("PropertyRead") => {
                    props_read_def.push(name.clone());
                }, "Properties" => {
                    props = Some(name.clone());
                }, _ => {}
                }
            }
            print!("\n");
            
            if let Some(props) = props {
                let code = quote!{
                    impl PropertiesExt for #struct_ident {
                        fn init_props() -> Self {
                            let prop_str = [#(stringify!(#props_read)),*];
                            let props = Properties::new(&prop_str);
                            Self {
                                #(#props_read_def: Default::default()), *,
                                #(#props_read: props.prop(stringify!(#props_read))), *,
                                #props: props,
                            }
                        }
                        fn get_props(&self) -> String {
                            let prop_str = [#(stringify!(#props_read)),*];
                            prop_str.join(", ").into()
                        }
//                         fn get_properties(&self) -> HashMap<String, PropertyRead<f32>> {
//                             self.props.clone()
//                         }
                    }
                };
//                 println!("Code: {}", code);
                return code.into();
            }
        }
    }
    TokenStream::new()
}
