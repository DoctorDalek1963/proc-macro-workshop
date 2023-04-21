use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    //dbg!(&input);

    let ident = input.ident.clone();
    let ident_span = ident.span();
    let builder_struct = format_ident!("{}Builder", ident);

    // Get the fields from the struct, or fail if it's a tuple or unit struct, or not a struct
    let fields = match input.data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(named_fields) => named_fields,
            syn::Fields::Unnamed(_) => {
                return syn::Error::new(
                    ident_span,
                    "can only derive Builder for structs with named fields (not tuple structs)",
                )
                .to_compile_error()
                .into();
            }
            syn::Fields::Unit => {
                return quote! {
                    impl #ident {
                        pub fn builder() -> #builder_struct {
                            #builder_struct {}
                        }
                    }

                    pub struct #builder_struct {}

                    impl #builder_struct {
                        pub fn build() -> #ident {
                            unimplemented!()
                        }
                    }
                }
                .into()
            }
        },
        _ => {
            return syn::Error::new(ident_span, "can only derive Builder for structs")
                .to_compile_error()
                .into();
        }
    };

    // The fields of the builder struct with each type wrapped in `Option<>`. For use in struct
    // declaration
    let builder_fields_with_types = fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.clone().expect(
                "We only derive Builder for structs with named fields, so the ident must exist",
            );
            let ty = field.ty.clone();
            quote! {
                #ident: ::std::option::Option<#ty>
            }
        })
        .collect::<Vec<_>>();

    // The fields of the builder struct each followed with `: None`. For use in the builder()
    // associated function on the original struct
    let builder_fields_colon_none = fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.clone().expect(
                "We only derive Builder for structs with named fields, so the ident must exist",
            );
            quote! {
                #ident: ::std::option::Option::None
            }
        })
        .collect::<Vec<_>>();

    let builder_setter_methods = fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.clone().expect(
                "We only derive Builder for structs with named fields, so the ident must exist",
            );
            let ty = field.ty.clone();
            quote! {
                pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = ::std::option::Option::Some(#ident);
                    self
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl #ident {
            pub fn builder() -> #builder_struct {
                #builder_struct {
                    #(#builder_fields_colon_none),*
                }
            }
        }

        pub struct #builder_struct {
            #(#builder_fields_with_types),*
        }

        impl #builder_struct {
            #(#builder_setter_methods)*

            pub fn build(self) -> #ident {
                unimplemented!()
            }
        }
    }
    .into()
}
