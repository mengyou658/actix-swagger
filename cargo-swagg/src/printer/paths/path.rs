use super::ResponseStatus;
use crate::printer::Printable;
use inflections::Inflect;
use quote::{format_ident, quote};

pub struct Path {
    pub name: String,
    pub response: ResponseEnum,
}

impl Path {
    fn print_enum_variants(&self) -> proc_macro2::TokenStream {
        let mut tokens = quote! {};

        for status in &self.response.responses {
            let variant = status.print_enum_variant();

            tokens = quote! {
                #tokens
                #variant,
            };
        }

        tokens
    }

    fn print_status_variants(&self) -> proc_macro2::TokenStream {
        let mut tokens = quote! {};

        for status in &self.response.responses {
            let variant = status.print_status_variant();

            tokens = quote! {
                #tokens
                #variant,
            };
        }

        quote! {
            match self {
                #tokens
            }
        }
    }

    fn print_content_type_variants(&self) -> proc_macro2::TokenStream {
        let mut tokens = quote! {};

        for status in &self.response.responses {
            let variant = status.print_content_type_variant();

            tokens = quote! {
                #tokens
                #variant,
            }
        }

        quote! {
            match self {
                #tokens
            }
        }
    }
}

impl Printable for Path {
    fn print(&self) -> proc_macro2::TokenStream {
        let module_name = format_ident!("{}", self.name.to_snake_case());
        let enum_variants = self.print_enum_variants();
        let status_match = self.print_status_variants();
        let content_type_match = self.print_content_type_variants();

        quote! {
            pub mod #module_name {
                use super::components::responses;
                use actix_swagger::{Answer, ContentType};
                use actix_web::http::StatusCode;
                use serde::Serialize;

                #[derive(Debug, Serialize)]
                #[serde(untagged)]
                pub enum Response {
                    #enum_variants
                }

                impl Response {
                    #[inline]
                    pub fn answer(self) -> Answer<'static, Self> {
                        let status = #status_match;
                        let content_type = #content_type_match;

                        Answer::new(self).status(status).content_type(content_type)
                    }
                }
            }
        }
    }
}

pub struct ResponseEnum {
    pub responses: Vec<StatusVariant>,
}

pub struct StatusVariant {
    pub status: ResponseStatus,

    /// Should be in `#/components/responses/`
    pub response_type_name: Option<String>,

    /// Comment for response status
    pub description: Option<String>,

    /// Now supports only one content type per response
    pub content_type: Option<ContentType>,

    /// Variant can be renamed with `x-variant-name`
    pub x_variant_name: Option<String>,
}

impl StatusVariant {
    pub fn name(&self) -> proc_macro2::Ident {
        let name = self
            .x_variant_name
            .clone()
            .unwrap_or(self.status.to_string());
        format_ident!("{}", name.to_pascal_case())
    }

    pub fn description(&self) -> proc_macro2::TokenStream {
        match &self.description {
            Some(text) => quote! { #[doc = #text] },
            None => quote! {},
        }
    }

    pub fn content_type(&self) -> proc_macro2::TokenStream {
        match self.content_type.clone() {
            Some(t) => {
                let content = t.print();
                quote! { Some(ContentType::#content) }
            }
            None => quote! { None },
        }
    }

    pub fn print_enum_variant(&self) -> proc_macro2::TokenStream {
        let description = self.description();
        let variant_name = self.name();

        if let Some(response) = self.response_type_name.clone() {
            let response_name = format_ident!("{}", response);

            quote! {
                #description
                #variant_name(responses::#response_name)
            }
        } else {
            quote! {
                #description
                #variant_name
            }
        }
    }

    pub fn print_status_variant(&self) -> proc_macro2::TokenStream {
        let variant_name = self.name();
        let status = format_ident!("{}", self.status.to_string().to_constant_case());

        if let Some(_) = self.response_type_name {
            quote! {
                Self::#variant_name(_) => StatusCode::#status
            }
        } else {
            quote! {
                Self::#variant_name => StatusCode::#status
            }
        }
    }

    pub fn print_content_type_variant(&self) -> proc_macro2::TokenStream {
        let variant_name = self.name();
        let content_type = self.content_type();

        if let Some(_) = self.response_type_name {
            quote! {
                Self::#variant_name(_) => #content_type
            }
        } else {
            quote! {
                Self::#variant_name => #content_type
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ContentType {
    Json,
}

impl Printable for ContentType {
    fn print(&self) -> proc_macro2::TokenStream {
        let ident = format_ident!(
            "{}",
            match self {
                Self::Json => "Json",
            }
        );

        quote! { #ident }
    }
}
