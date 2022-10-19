use proc_macro::TokenStream;
use syn::{
    Ident,
    __private::{quote::quote, Span, TokenStream2},
    parse_macro_input, DeriveInput, Fields, ItemStruct,
};

#[proc_macro_attribute]
pub fn index_list_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let ty = Ident::new(&attr.to_string(), Span::call_site());
    let struct_ty = input.ident.clone();

    let names = input
        .fields
        .clone()
        .into_iter()
        .map(|e| {
            let name = e.ident.unwrap();
            quote! { &self.#name }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #input

        impl cw_storage_plus::IndexList<#ty> for #struct_ty<'_> {
            fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn cw_storage_plus::Index<#ty>> + '_> {
                let v: Vec<&dyn cw_storage_plus::Index<#ty>> = vec![#(#names),*];
                Box::new(v.into_iter())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(StorageKey)]
pub fn derive_storage_key(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let ident = input.ident;

    match input.data {
        syn::Data::Enum(data_enum) => {
            let (b, b_rev): (Vec<TokenStream2>, Vec<TokenStream2>) = data_enum
                .variants
                .into_iter()
                .enumerate()
                .map(|e| {
                    let id = e.1.ident;
                    let data = match e.1.fields {
                        Fields::Unit => e.0 as u8,
                        _ => panic!(
                            "#[derive(StorageKey)] currently only support unit enum variants"
                        ),
                    };

                    (
                        quote! {
                            Self::#id => #data
                        },
                        quote! {
                            #data => Self::#id
                        },
                    )
                })
                .unzip();

            let expanded = quote! {
                impl #ident {
                    fn as_bytes(&self) -> [u8; 1] {
                        match self { #(#b),* }.to_be_bytes()
                    }

                    fn from_slice(b: &[u8]) -> Self {
                        match b[0] {
                            #(#b_rev),*,
                            _ => panic!("Should not be others")
                        }
                    }
                }

                impl cw_storage_plus::PrimaryKey<'_> for #ident {
                    type Prefix = ();
                    type SubPrefix = ();
                    type Suffix = ();
                    type SuperSuffix = ();

                    fn key(&self) -> Vec<cw_storage_plus::Key> {
                        vec![cw_storage_plus::Key::Val8(self.as_bytes())]
                    }
                }

                impl<'a> cw_storage_plus::PrimaryKey<'a> for &'a #ident {
                    type Prefix = ();
                    type SubPrefix = ();
                    type Suffix = ();
                    type SuperSuffix = ();

                    fn key(&self) -> Vec<cw_storage_plus::Key> {
                        vec![cw_storage_plus::Key::Val8(self.as_bytes())]
                    }
                }

                impl cw_storage_plus::Prefixer<'_> for #ident {
                    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
                        vec![cw_storage_plus::Key::Val8(self.as_bytes())]
                    }
                }

                impl<'a> cw_storage_plus::Prefixer<'a> for &'a #ident {
                    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
                        vec![cw_storage_plus::Key::Val8(self.as_bytes())]
                    }
                }

                impl cw_storage_plus::KeyDeserialize for #ident {
                    type Output = Self;

                    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
                        Ok(#ident::from_slice(&value))
                    }
                }
            };

            TokenStream::from(expanded)
        }
        _ => panic!("#[derive(StorageKey)] currently only support enums."),
    }
}
