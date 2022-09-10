use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Fields, GenericArgument, Ident,
    PathArguments, Type,
};

struct FieldProperties {
    ident: Ident,
    ty: Type,
}

#[proc_macro_derive(SectionParser)]
pub fn derive_section_parser(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident: struct_ident,
        data,
        generics,
        ..
    } = parse_macro_input!(item);
    if !generics.params.is_empty() {
        panic!("SectionParser does not support generics yet");
    }

    let DataStruct { fields, .. } = match data {
        Data::Struct(data) => data,
        _ => panic!("SectionParser can only be derived on structs"),
    };
    let fields = match fields {
        Fields::Named(fields) => fields,
        _ => panic!("SectionParser can only be derived on structs with named fields"),
    };

    let error_ident = Ident::new(&format!("{}Error", struct_ident), Span::call_site());

    let mut parsed_fields = Vec::new();

    for field in fields.named {
        let ident = field.ident.expect("Expected named field");
        let path = match field.ty {
            Type::Path(path) => path.path.segments,
            _ => continue,
        };
        let first_segment = path
            .into_iter()
            .next()
            .expect("Expected type with at least one element");
        if first_segment.ident != "Option" {
            continue;
        }
        let option_type_arguments = match first_segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments.args,
            _ => panic!(
                "Expected angle bracketed arguments for Option, but got {:?}",
                first_segment.arguments
            ),
        };
        let type_argument = option_type_arguments
            .into_iter()
            .next()
            .expect("Expected one type argument for Option");
        let ty = match type_argument {
            GenericArgument::Type(ty) => ty,
            _ => panic!(
                "Expected a type argument for Option, but got {:?}",
                type_argument
            ),
        };

        parsed_fields.push(FieldProperties { ident, ty });
    }

    let getters: TokenStream = parsed_fields
        .iter()
        .map(|FieldProperties { ident, ty, .. }| {
            quote! {
                fn #ident(&mut self) -> ::std::result::Result<#ty, #error_ident> {
                    self.#ident.take().ok_or_else(|| {
                        self.missing_field_error(stringify!(#ident))
                    })
                }
            }
        })
        .collect();

    let setters: TokenStream = parsed_fields.iter().map(|FieldProperties {ident, ty, .. }| {
        let setter = Ident::new(&format!("set_{}", ident), Span::call_site());
        quote! {
            fn #setter(&mut self, #ident: #ty) -> ::std::result::Result<(), #error_ident> {
                if let Some(#ident) = self.#ident.take() {
                    ::std::result::Result::Err(self.duplicate_field_error(stringify!(#ident), #ident))
                } else {
                    self.#ident = ::std::option::Option::Some(#ident);
                    ::std::result::Result::Ok(())
                }
            }
        }
    }).collect();

    let ensure_empty_checks: TokenStream = parsed_fields.iter().map(|FieldProperties { ident, .. }| {
        quote! {
            if let ::std::option::Option::Some(#ident) = self.#ident.take() {
                return ::std::result::Result::Err(self.unexpected_field_error(stringify!(#ident), #ident));
            }
        }
    }).collect();

    let result = quote! {
        impl #struct_ident {
            #getters

            #setters

            fn ensure_empty(mut self) -> ::std::result::Result<(), #error_ident> {
                #ensure_empty_checks

                ::std::result::Result::Ok(())
            }
        }
    };
    result.into()
}
