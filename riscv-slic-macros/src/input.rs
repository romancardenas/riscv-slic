use syn::parse::Parse;
use syn::{bracketed, parse::ParseStream, token::Comma, Error, Ident, Path, Result, Token};

pub use crate::export::ExportBackendInput; // backend-specific input

pub struct HandlersInput(Vec<Ident>);

impl core::ops::Deref for HandlersInput {
    type Target = Vec<Ident>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for HandlersInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);
        let idents = content.parse_terminated(Ident::parse, Comma)?;
        Ok(Self(idents.into_iter().collect()))
    }
}

pub struct CodegenInput {
    pub slic: Path,
    pub pac: Path,
    pub swi_handlers: HandlersInput,
    #[allow(dead_code)]
    pub backend: Option<ExportBackendInput>,
}

impl Parse for CodegenInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut slic = None;
        let mut pac = None;
        let mut swi_handlers = None;
        let mut backend = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "slic" => {
                    if slic.is_some() {
                        return Err(Error::new(ident.span(), "duplicate identifier"));
                    }
                    input.parse::<Token![=]>()?; // consume the '='
                    slic = Some(input.parse()?);
                }
                "pac" => {
                    if pac.is_some() {
                        return Err(Error::new(ident.span(), "duplicate identifier"));
                    }
                    input.parse::<Token![=]>()?; // consume the '='
                    pac = Some(input.parse()?);
                }
                "swi" => {
                    if swi_handlers.is_some() {
                        return Err(Error::new(ident.span(), "duplicate identifier"));
                    }
                    input.parse::<Token![=]>()?; // consume the '='
                    swi_handlers = Some(input.parse()?);
                }
                "backend" => {
                    if backend.is_some() {
                        return Err(Error::new(ident.span(), "duplicate identifier"));
                    }
                    input.parse::<Token![=]>()?; // consume the '='
                    backend = Some(input.parse()?);
                }
                _ => return Err(Error::new(ident.span(), "invalid identifier")),
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?; // consume the ',' between identifiers
            }
        }

        let slic = match slic {
            Some(slic) => slic,
            None => syn::parse_str("riscv_slic").unwrap(),
        };

        Ok(CodegenInput {
            slic,
            pac: pac.ok_or_else(|| Error::new(input.span(), "missing identifier"))?,
            swi_handlers: swi_handlers
                .ok_or_else(|| Error::new(input.span(), "missing identifier"))?,
            backend,
        })
    }
}
