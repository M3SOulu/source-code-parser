use darling::ToTokens;
use proc_macro2::TokenStream; //Span,
use quote::quote; //format_ident,
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields, GenericArgument, Ident,
    PathArguments, PathSegment, Type,
};

macro_rules! match_or {
    ($pattern:pat, $name:ident, $thing:expr) => {
        if let $pattern = $thing {
            Some($name)
        } else {
            None
        }
    };
}

pub fn get_inner_path_segment(path_segment: &PathSegment) -> Option<PathSegment> {
    let args = match_or!(
        PathArguments::AngleBracketed(ref args),
        args,
        path_segment.arguments
    )?;
    let generic_arg = args.args.first()?;
    let type_path = match_or!(GenericArgument::Type(Type::Path(path)), path, generic_arg)?;

    Some(type_path.path.segments.first()?.clone())
}

pub fn get_struct_fields(r#struct: &DataStruct) -> Vec<(&Ident, &Type)> {
    match &r#struct.fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().expect("You're named for a reason.");
                let ty = &field.ty;
                (ident, ty)
            })
            .collect(),
        _ => unimplemented!(),
    }
}

pub fn get_enum_variants(r#enum: &DataEnum) -> Vec<(&Ident, &Type)> {
    r#enum
        .variants
        .iter()
        .map(|variant| {
            let ty = match &variant.fields {
                Fields::Unnamed(fields) => fields.unnamed.first().map(|field| &field.ty),
                _ => unimplemented!(), // assuming tuple variants
            }
            .expect("There should be at least one unnamed field");
            (&variant.ident, ty)
        })
        .collect()
}