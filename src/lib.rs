use proc_macro::{TokenStream};
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, Data, DataStruct, DeriveInput, Fields, Type, PathArguments, GenericArgument};

#[proc_macro_derive(SectionParser)]
pub fn derive_section_parser(item: TokenStream) -> TokenStream {
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

    let error_ident = Ident::new(
        &format!("{}Error", struct_ident.to_string()),
        Span::call_site(),
    );

    let mut parsed_fields = Vec::new();
    let mut errors = Vec::new();

    for field in fields.named {
        let ident = field
            .ident
            .expect("Expected named field");
        let path = match field.ty {
            Type::Path(path) => path.path.segments,
            _ => continue,
        };
        let first_segment = path.into_iter().next().expect("Expected type with at least one element");
        if first_segment.ident.to_string() != "Option" {
            continue;
        }
        let option_type_arguments = match first_segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments.args,
            _ => panic!("Expected angle bracketed arguments for Option, but got {:?}", first_segment.arguments),
        };
        let type_argument = option_type_arguments.into_iter().next().expect("Expected one type argument for Option");
        let ty = match type_argument {
            GenericArgument::Type(ty) => ty,
            _ => panic!("Expected a type argument for Option, but got {:?}", type_argument),
        };

        let camelcase_ident = ident.to_string().to_case(Case::Camel);
        let unexpected_error = format!("Unexpected{camelcase_ident}");
        let duplicate_error = format!("Duplicate{camelcase_ident}");
        let missing_error = format!("Missing{camelcase_ident}");

        let setter = Ident::new(&format!("set_{}", ident.to_string()), Span::call_site());

        output.push(quote!{
            impl #struct_ident {
                fn #setter(&mut self, #ident: #ty) -> ::std::result::Result<(), #error_ident> {
                    if self.#ident.is_none() {
                        self.#ident = ::std::option::Option::Some(#ident);
                        ::std::result::Result::Ok(())
                    } else {
                        ::std::result::Result::Err(#error_ident::#unexpected_error)
                    }
                }

                fn #ident(&mut self) -> ::std::result::Result<#ty, #error_ident> {
                    self.#ident.take().ok_or_else(|| {
                        #error_ident::#missing_error
                    })
                }
            }
        });

        errors.extend([unexpected_error, duplicate_error, missing_error]);
    }

    todo!()
}
