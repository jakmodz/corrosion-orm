use corrosion_orm_core::validation::{Validation, ValidationType};
use syn::{DeriveInput, Field, spanned::Spanned};

use crate::validation_parser::{
    validation_attributes::{EmailAttribute, NotNullAttribute, PatternAttribute, SizeAttribute},
    validation_field::ValidationRule,
};

pub(crate) fn parse_validation(ast: &mut DeriveInput) -> syn::Result<Vec<ValidationRule>> {
    if let syn::Data::Struct(data) = &mut ast.data {
        let mut all_rules = Vec::new();
        for field in data.fields.iter_mut() {
            all_rules.extend(parse_field_validations(field)?);
        }
        return Ok(all_rules);
    }
    Err(syn::Error::new(
        ast.span(),
        "Validation can only be derived on structs",
    ))
}

fn parse_field_validations(f: &mut Field) -> syn::Result<Vec<ValidationRule>> {
    let mut rules = Vec::new();
    let ident = f
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new(f.span(), "Field must have an identifier"))?
        .clone();

    if f.attrs.iter().any(|a| a.path().is_ident("NotNull")) {
        let a = deluxe::extract_attributes::<_, NotNullAttribute>(&mut f.attrs)?;
        rules.push(ValidationRule::new(
            ident.clone(),
            Validation::new(ValidationType::NotNull, a.message.unwrap_or_default()),
        ));
    }

    if f.attrs.iter().any(|a| a.path().is_ident("Size")) {
        if !is_string_type(&f.ty) {
            return Err(syn::Error::new(
                f.ty.span(),
                "#[Size] is only supported for String types",
            ));
        }
        let a = deluxe::extract_attributes::<_, SizeAttribute>(&mut f.attrs)?;
        rules.push(ValidationRule::new(
            ident.clone(),
            Validation::new(
                ValidationType::Size {
                    min: a.min,
                    max: a.max,
                },
                a.message.unwrap_or_default(),
            ),
        ));
    }

    if f.attrs.iter().any(|a| a.path().is_ident("Pattern")) {
        if !is_string_type(&f.ty) {
            return Err(syn::Error::new(
                f.ty.span(),
                "#[Pattern] is only supported for String types",
            ));
        }
        let a = deluxe::extract_attributes::<_, PatternAttribute>(&mut f.attrs)?;
        rules.push(ValidationRule::new(
            ident.clone(),
            Validation::new(
                ValidationType::Regex { pattern: a.pattern },
                a.message.unwrap_or_default(),
            ),
        ));
    }

    if f.attrs.iter().any(|a| a.path().is_ident("Email")) {
        if !is_string_type(&f.ty) {
            return Err(syn::Error::new(
                f.ty.span(),
                "#[Email] is only supported for String types",
            ));
        }
        let a = deluxe::extract_attributes::<_, EmailAttribute>(&mut f.attrs)?;
        rules.push(ValidationRule::new(
            ident.clone(),
            Validation::new(ValidationType::Email, a.message.unwrap_or_default()),
        ));
    }

    Ok(rules)
}

fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        return tp.path.segments.last().is_some_and(|s| s.ident == "String");
    }
    false
}
