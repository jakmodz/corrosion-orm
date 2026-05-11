use syn::{Type, TypePath};

pub(crate) fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.first()
    {
        return segment.ident == "Option";
    }
    false
}
pub fn type_is_ident(ty: &Type, name: &str) -> bool {
    let ty = strip_wrappers(ty);
    match ty {
        Type::Path(TypePath { qself: None, path }) => path
            .segments
            .last()
            .map(|seg| seg.ident == name)
            .unwrap_or(false),
        _ => false,
    }
}
fn strip_wrappers(mut ty: &Type) -> &Type {
    loop {
        match ty {
            Type::Reference(r) => ty = &*r.elem,
            Type::Paren(p) => ty = &*p.elem,
            Type::Group(g) => ty = &*g.elem,
            _ => break,
        }
    }
    ty
}
/// Extracts the inner type `T` from a `syn::Type` if it is a `syn::Type::Path`.
/// Otherwise, returns the original type unchanged.
/// # Examples
///
/// ```ignore
/// use syn::Type;
/// use quote::ToTokens;
///
/// // Vec inner type is extracted
/// let vec_ty: Type = syn::parse_str("Vec<i32>").unwrap();
/// let inner = extract_inner_type(&vec_ty);
/// assert_eq!(inner.to_token_stream().to_string(), "i32");
///
/// // Non-Vec types are returned unchanged (cloned)
/// let simple_ty: Type = syn::parse_str("String").unwrap();
/// let same = extract_inner_type(&simple_ty);
/// assert_eq!(same.to_token_stream().to_string(), "String");
/// ```
pub fn extract_inner_type(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return inner_ty.clone();
    }

    ty.clone()
}

pub fn extract_type_ident(ty: &Type) -> Option<syn::Ident> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return Some(segment.ident.clone());
    }
    None
}
