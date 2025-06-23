use syn::{
    Data, DeriveInput, Error, Ident, ItemFn, Path, Result, ReturnType, Token, Type, Visibility,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

pub use crate::export::ExportBackendInput; // backend-specific input

pub struct SwiAttr {
    pub slic: Path,
    pub pac: Path,
    pub backend: ExportBackendInput,
}

impl Parse for SwiAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut slic = None;
        let mut pac = None;
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

        Ok(Self {
            slic: slic.unwrap_or_else(|| syn::parse_str("riscv_slic").unwrap()),
            pac: pac.ok_or_else(|| Error::new(input.span(), "missing identifier"))?,
            backend: backend.unwrap_or_else(|| ExportBackendInput::default()),
        })
    }
}

pub struct SwiItem {
    /// The name of the enum
    pub name: Ident,
    /// A map from discriminant values to variant names
    pub sources: Vec<Ident>,
}

impl Parse for SwiItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let input: DeriveInput = input.parse()?;
        let variants = match input.data {
            Data::Enum(data) => Ok(data.variants),
            _ => Err(Error::new(input.span(), "Only enums are supported")),
        }?;

        let name = input.ident.clone();
        let mut sources = Vec::new();
        for v in variants.iter() {
            if !v.fields.is_empty() {
                return Err(Error::new(
                    v.span(),
                    "Interrupt source variants must not have fields",
                ));
            }
            sources.push(v.ident.clone());
        }

        Ok(Self { name, sources })
    }
}

pub struct InterruptAttr {
    pub source: Path,
    pub slic: Path,
}

impl Parse for InterruptAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        // source is mandartory and appears first
        let source: Path = input.parse()?;
        let mut slic = None;

        if !input.is_empty() {
            input.parse::<Token![,]>()?; // consume the ',' between identifiers

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
                    _ => return Err(Error::new(ident.span(), "invalid identifier")),
                }
                if !input.is_empty() {
                    input.parse::<Token![,]>()?; // consume the ',' between identifiers
                }
            }
        }

        Ok(Self {
            source,
            slic: slic.unwrap_or_else(|| syn::parse_str("riscv_slic").unwrap()),
        })
    }
}

pub struct InterruptItem(pub ItemFn);

impl Parse for InterruptItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let item: ItemFn = input.parse()?;

        if !item.sig.inputs.is_empty() {
            return Err(Error::new(
                item.sig.inputs.span(),
                "Interrupt handler must have no arguments",
            ));
        }
        if item.sig.constness.is_some() {
            return Err(Error::new(
                item.sig.constness.span(),
                "Interrupt handler must not be const",
            ));
        }
        if item.sig.asyncness.is_some() {
            return Err(Error::new(
                item.sig.asyncness.span(),
                "Interrupt handler must not be async",
            ));
        }
        if !matches!(item.vis, Visibility::Inherited) {
            return Err(Error::new(
                item.vis.span(),
                "Interrupt handler must be private",
            ));
        }
        if item.sig.abi.is_some() {
            return Err(Error::new(
                item.sig.abi.span(),
                "Interrupt handler must not have an ABI",
            ));
        }
        if !item.sig.generics.params.is_empty() {
            return Err(Error::new(
                item.sig.generics.params.span(),
                "Interrupt handler must not have generics",
            ));
        }
        if item.sig.generics.where_clause.is_some() {
            return Err(Error::new(
                item.sig.generics.where_clause.span(),
                "Interrupt handler must not have a where clause",
            ));
        }
        if item.sig.variadic.is_some() {
            return Err(Error::new(
                item.sig.variadic.span(),
                "Interrupt handler must not be variadic",
            ));
        }
        if let ReturnType::Type(_, ref ty) = item.sig.output {
            if !matches!(**ty, Type::Never(_)) {
                return Err(Error::new(
                    item.sig.output.span(),
                    "Interrupt handler can only return !",
                ));
            }
        }

        Ok(Self(item))
    }
}
