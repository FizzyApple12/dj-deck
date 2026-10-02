use proc_macro::TokenStream;
use syn::{DeriveInput, Fields};
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(EnumStrConverters)]
pub fn make_enum_str_converters(input: TokenStream) -> TokenStream {
    let syn_item: syn::DeriveInput = syn::parse_macro_input!(input as DeriveInput);

    let enum_name = syn_item.ident;
    let syn::Data::Enum(enum_item) = syn_item.data else {
        panic!("EnumStrConverters only works on enums")
    };

    let to_str_arms = enum_item.variants.iter().map(|variant| {
        let vname = &variant.ident;
        let vname_string = vname.to_string();

        let pattern = match &variant.fields {
            Fields::Unit => quote! { #enum_name::#vname },
            Fields::Unnamed(_) => quote! { #enum_name::#vname(..) },
            Fields::Named(_) => quote! { #enum_name::#vname { .. } },
        };

        quote! { #pattern => #vname_string }
    });

    let from_str_arms = enum_item.variants.iter().map(|variant| {
        let vname = &variant.ident;
        let vname_string = vname.to_string();

        let initializer = match &variant.fields {
            Fields::Unit => quote! { #enum_name::#vname },
            Fields::Unnamed(fields) => {
                let initializers = fields.unnamed.iter().map(|_| {
                    quote! {
                        ::core::default::Default::default()
                    }
                });

                quote! { #enum_name::#vname(#(#initializers),*) }
            }
            Fields::Named(fields) => {
                let initializers = fields.named.iter().filter_map(|field| {
                    let field_name = field.ident.clone()?;

                    Some(quote! {
                        #field_name: ::core::default::Default::default()
                    })
                });

                quote! { #enum_name::#vname { #(#initializers),* } }
            }
        };

        quote! { #vname_string => { ::core::option::Option::Some(#initializer) } }
    });

    let expanded = quote! {
        impl #enum_name {
            pub fn to_str(action: #enum_name) -> &'static str {
                match action {
                    #(#to_str_arms),*
                }
            }

            pub fn from_str(string: &str) -> ::core::option::Option<#enum_name> {
                match string {
                    #(#from_str_arms),*
                     _ => ::core::option::Option::None
                }
            }
        }
    };
    expanded.into()
}
