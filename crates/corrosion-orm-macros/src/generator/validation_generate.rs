use crate::validation_parser::validation_field::ValidationRule;
use corrosion_orm_core::validation::ValidationType;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::Ident;

pub(crate) fn generate_validations(name: &Ident, validations: Vec<ValidationRule>) -> TokenStream {
    let validations_impl: Vec<TokenStream> = validations
        .iter()
        .map(generate_validation_from_type)
        .collect();

    quote! {
        impl #name {
            pub fn validate(&self) -> Result<(), corrosion_orm_core::validation::ValidationError> {
                #(#validations_impl)*
                Ok(())
            }
        }
    }
}

fn generate_validation_from_type(validation: &ValidationRule) -> TokenStream {
    let ident = &validation.ident;
    let field_name = validation.ident.to_string();
    let msg_literal = &validation.validation.message;
    let msg = if !msg_literal.is_empty() {
        quote! { Some(#msg_literal) }
    } else {
        quote! { None }
    };

    match &validation.validation.ty {
        ValidationType::NotNull => {
            quote! {
                {
                    use corrosion_orm_core::validation::{NotNullValidator, Validator};
                    let validator = NotNullValidator;
                    validator.validate(#field_name, &self.#ident, #msg)?;
                }
            }
        }
        ValidationType::Size { min, max } => {
            let min_tokens = match min {
                Some(m) => quote! { Some(#m) },
                None => quote! { None },
            };
            let max_tokens = match max {
                Some(m) => quote! { Some(#m) },
                None => quote! { None },
            };

            quote! {
                {
                    use corrosion_orm_core::validation::{SizeValidator, Validator};
                    let validator = SizeValidator { min: #min_tokens, max: #max_tokens };
                    validator.validate(#field_name, &self.#ident, #msg)?;
                }
            }
        }
        ValidationType::Regex { pattern } => {
            let pattern_lit = Literal::string(pattern);
            quote! {
                {
                    use corrosion_orm_core::validation::{PatternValidator, Validator};
                    static VALIDATOR: corrosion_orm_core::once_cell::sync::Lazy<PatternValidator> =
                        corrosion_orm_core::once_cell::sync::Lazy::new(|| {
                            PatternValidator::new(#pattern_lit)
                                .expect("Regex pattern was validated at compile time")
                        });
                    VALIDATOR.validate(#field_name, &self.#ident, #msg)?;
                }
            }
        }
        ValidationType::Email => {
            let email_pattern = r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";
            quote! {
                {
                    use corrosion_orm_core::validation::{PatternValidator, Validator};
                    static VALIDATOR: corrosion_orm_core::once_cell::sync::Lazy<PatternValidator> =
                        corrosion_orm_core::once_cell::sync::Lazy::new(|| {
                            PatternValidator::new(#email_pattern).expect("Email regex should be valid")
                        });
                    VALIDATOR.validate(#field_name, &self.#ident, #msg)?;
                }
            }
        }
    }
}
