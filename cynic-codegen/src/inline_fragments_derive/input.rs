use {darling::util::SpannedValue, proc_macro2::Span, quote::quote_spanned};

use crate::error::Errors;

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(enum_newtype, enum_unit))]
pub struct InlineFragmentsDeriveInput {
    pub(super) ident: proc_macro2::Ident,
    pub(super) data: darling::ast::Data<SpannedValue<InlineFragmentsDeriveVariant>, ()>,
    pub(super) generics: syn::Generics,

    pub schema_path: SpannedValue<String>,

    #[darling(default, rename = "schema_module")]
    schema_module_: Option<syn::Path>,

    #[darling(default)]
    pub graphql_type: Option<SpannedValue<String>>,

    // argument_struct is deprecated, remove eventually.
    #[darling(default)]
    argument_struct: Option<syn::Ident>,
    #[darling(default)]
    variables: Option<syn::Path>,
}

impl InlineFragmentsDeriveInput {
    pub fn schema_module(&self) -> syn::Path {
        if let Some(schema_module) = &self.schema_module_ {
            return schema_module.clone();
        }
        syn::parse2(quote::quote! { schema }).unwrap()
    }

    pub fn graphql_type_name(&self) -> String {
        self.graphql_type
            .as_ref()
            .map(|sp| sp.to_string())
            .unwrap_or_else(|| self.ident.to_string())
    }

    pub fn graphql_type_span(&self) -> Span {
        self.graphql_type
            .as_ref()
            .map(|val| val.span())
            .unwrap_or_else(|| self.ident.span())
    }

    pub fn variables(&self) -> Option<syn::Path> {
        self.variables
            .clone()
            .or_else(|| self.argument_struct.clone().map(Into::into))
    }

    pub(super) fn validate(&self, mode: ValidationMode) -> Result<(), Errors> {
        let data_ref = self.data.as_ref().take_enum().unwrap();

        let fallbacks = data_ref.iter().filter(|v| *v.fallback).collect::<Vec<_>>();
        let mut errors = Errors::default();

        if fallbacks.is_empty() {
            errors.push(syn::Error::new(proc_macro2::Span::call_site(), "InlineFragments derives require a fallback.  Add a unit variant and mark it with `#[cynic(fallback)]`"));
        }

        if fallbacks.len() > 1 {
            errors.extend(
                fallbacks
                    .into_iter()
                    .map(|f| {
                        syn::Error::new(
                    f.span(),
                    "InlineFragments only support a single fallback, but this enum has many",
                )
                    })
                    .collect::<Vec<_>>(),
            );
        }

        errors.extend(
            data_ref
                .iter()
                .filter_map(|v| v.validate(mode, v.span()).err()),
        );

        errors.into_result(())
    }

    pub fn deprecations(&self) -> proc_macro2::TokenStream {
        let mut rv = proc_macro2::TokenStream::new();

        if self.variables.is_none() && self.argument_struct.is_some() {
            let span = self.argument_struct.as_ref().map(|x| x.span()).unwrap();
            rv.extend(quote_spanned! { span =>
                #[allow(clippy::no_effect, non_camel_case_types)]
                const _: fn() = || {
                    #[deprecated(note = "the argument_struct attribute is deprecated.  use the variables attribute instead", since = "2.0.0")]
                    struct argument_struct {}
                    argument_struct {};
                };
            });
        }

        let data_ref = self.data.as_ref().take_enum().unwrap();
        for variant in data_ref {
            rv.extend(variant.deprecations());
        }

        rv
    }
}

#[derive(darling::FromVariant)]
#[darling(attributes(cynic))]
pub(super) struct InlineFragmentsDeriveVariant {
    pub(super) ident: proc_macro2::Ident,
    pub fields: darling::ast::Fields<InlineFragmentsDeriveField>,

    #[deprecated(
        note = "rename on an InlineFragments variant is deprecated",
        since = "2.0.0"
    )]
    #[darling(default)]
    rename: Option<SpannedValue<String>>,

    #[darling(default)]
    pub(super) fallback: SpannedValue<bool>,
}

#[derive(darling::FromField)]
#[darling(attributes(cynic))]
pub(super) struct InlineFragmentsDeriveField {
    pub ty: syn::Type,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ValidationMode {
    Interface,
    Union,
}

impl InlineFragmentsDeriveVariant {
    fn validate(&self, mode: ValidationMode, span: proc_macro2::Span) -> Result<(), Errors> {
        use {
            darling::ast::Style::{Struct, Tuple, Unit},
            ValidationMode::{Interface, Union},
        };

        if let Struct = self.fields.style {}

        if *self.fallback {
            match (mode, self.fields.style, self.fields.len()) {
                (_, Unit, _) => Ok(()),
                (Interface | Union, Tuple, 1) => Ok(()),
                (_, Struct, _) => Err(syn::Error::new(
                    span,
                    "The InlineFragments derive doesn't currently support struct variants",
                )
                .into()),
                (Interface, Tuple, _) => Err(syn::Error::new(
                    span,
                    "InlineFragments fallbacks on an interface must be a unit or newtype variant",
                )
                .into()),
                (Union, Tuple, _) => Err(syn::Error::new(
                    span,
                    "InlineFragments fallbacks on a union must be a unit or newtype variant",
                )
                .into()),
            }
        } else {
            match (self.fields.style, self.fields.len()) {
                (Tuple, 1) => Ok(()),
                (Struct, _) => Err(syn::Error::new(
                    span,
                    "The InlineFragments derive doesn't currently support struct variants",
                )
                .into()),
                (_, _) => Err(syn::Error::new(
                    span,
                    "Variants on the InlineFragments derive should have one field",
                )
                .into()),
            }
        }
    }

    pub fn deprecations(&self) -> proc_macro2::TokenStream {
        #[allow(deprecated)]
        if self.rename.is_some() {
            let span = self.rename.as_ref().unwrap().span();
            return quote_spanned! { span =>
                #[allow(clippy::no_effect, non_camel_case_types)]
                const _: fn() = || {
                    #[deprecated(
                        note = "the rename attribute on InlineFragments is deprecated and should be removed.  InlineFragments variants now get their type name from their inner QueryFragment.",
                        since = "2.0.0"
                    )]
                    struct rename {}
                    rename {};
                };
            };
        }

        proc_macro2::TokenStream::new()
    }
}

#[cfg(test)]
mod tests {
    use {darling::FromDeriveInput, syn::parse_quote};

    use super::*;

    #[test]
    fn test_interface_validation() {
        let input = InlineFragmentsDeriveInput::from_derive_input(&parse_quote! {
            #[cynic(schema_path = "whatever")]
            enum TestStruct {
                AnIncorrectUnitVariant,
                AVariantThatIsFine(SomeStruct),
            }
        })
        .unwrap();

        insta::assert_display_snapshot!(input.validate(ValidationMode::Interface).unwrap_err(), @r###"
        InlineFragments derives require a fallback.  Add a unit variant and mark it with `#[cynic(fallback)]`
        Variants on the InlineFragments derive should have one field
        "###);
    }

    #[test]
    fn test_union_validation() {
        let input = InlineFragmentsDeriveInput::from_derive_input(&parse_quote! {
            #[cynic(schema_path = "whatever")]
            enum TestStruct {
                AnIncorrectUnitVariant,
                AVariantThatIsFine(SomeStruct),
            }
        })
        .unwrap();

        insta::assert_display_snapshot!(input.validate(ValidationMode::Union).unwrap_err(), @r###"
        InlineFragments derives require a fallback.  Add a unit variant and mark it with `#[cynic(fallback)]`
        Variants on the InlineFragments derive should have one field
        "###);
    }

    #[test]
    fn test_multiple_fallback_validation() {
        let input = InlineFragmentsDeriveInput::from_derive_input(&parse_quote! {
            #[cynic(schema_path = "whatever")]
            enum TestStruct {
                #[cynic(fallback)]
                FirstFallback,
                #[cynic(fallback)]
                SecondFallback,
            }
        })
        .unwrap();

        insta::assert_display_snapshot!(input.validate(ValidationMode::Union).unwrap_err(), @r###"
        InlineFragments only support a single fallback, but this enum has many
        InlineFragments only support a single fallback, but this enum has many
        "###);
    }

    #[test]
    fn test_interface_fallback_validation_happy_path() {
        let input = InlineFragmentsDeriveInput::from_derive_input(&parse_quote! {
            #[cynic(schema_path = "whatever")]
            enum TestStruct {
                #[cynic(fallback)]
                FirstFallback,
            }
        })
        .unwrap();

        input.validate(ValidationMode::Interface).unwrap();

        let input = InlineFragmentsDeriveInput::from_derive_input(&parse_quote! {
            #[cynic(schema_path = "whatever")]
            enum TestStruct {
                #[cynic(fallback)]
                FirstFallback(SomeStruct),
            }
        })
        .unwrap();

        input.validate(ValidationMode::Interface).unwrap();
    }

    #[test]
    fn test_union_fallback_validation_happy_path() {
        let input = InlineFragmentsDeriveInput::from_derive_input(&parse_quote! {
            #[cynic(schema_path = "whatever")]
            enum TestStruct {
                #[cynic(fallback)]
                FirstFallback,
            }
        })
        .unwrap();

        input.validate(ValidationMode::Union).unwrap();
    }

    #[test]
    fn test_union_fallback_validation_with_newtype() {
        let input = InlineFragmentsDeriveInput::from_derive_input(&parse_quote! {
            #[cynic(schema_path = "whatever")]
            enum TestStruct {
                #[cynic(fallback)]
                FirstFallback(String),
            }
        })
        .unwrap();

        input.validate(ValidationMode::Union).unwrap();
    }
}
