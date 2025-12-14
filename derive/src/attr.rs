use proc_macro2::Ident;
use syn::{Attribute, DeriveInput, Field, LitStr, Path, Result, Variant};

pub struct FieldAttrs {
    pub rename: Option<String>,
    pub skip_serializing_if: Option<Path>,
    pub default: Default,
}

pub struct ContainerAttrs {
    pub default: Default,
}

pub enum Default {
    None,
    Default,
    Path(Path),
}

pub fn get(field: &Field) -> Result<FieldAttrs> {
    let mut rename = None;
    let mut skip_serializing_if = None;
    let mut default = Default::None;

    for attr in &field.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let s: LitStr = meta.value()?.parse()?;
                if rename.is_some() {
                    return Err(meta.error("duplicate rename attribute"));
                }
                rename = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("skip_serializing_if") {
                let s: LitStr = meta.value()?.parse()?;
                if skip_serializing_if.is_some() {
                    return Err(meta.error("duplicate skip_serializing_if attribute"));
                }
                skip_serializing_if = Some(s.parse()?);
                Ok(())
            } else if meta.path.is_ident("default") {
                if !matches!(default, Default::None) {
                    return Err(meta.error("duplicate default attribute"));
                }
                if meta.input.is_empty() || meta.input.peek(syn::Token![,]) {
                    default = Default::Default;
                } else {
                    let s: LitStr = meta.value()?.parse()?;
                    default = Default::Path(s.parse()?);
                }
                Ok(())
            } else {
                Err(meta.error("unsupported attribute"))
            }
        })?;
    }

    Ok(FieldAttrs {
        rename,
        skip_serializing_if,
        default,
    })
}

pub fn get_container(input: &DeriveInput) -> Result<ContainerAttrs> {
    let mut default = Default::None;

    for attr in &input.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                if !matches!(default, Default::None) {
                    return Err(meta.error("duplicate default attribute"));
                }
                if meta.input.is_empty() || meta.input.peek(syn::Token![,]) {
                    default = Default::Default;
                } else {
                    let s: LitStr = meta.value()?.parse()?;
                    default = Default::Path(s.parse()?);
                }
                Ok(())
            } else {
                // We ignore other container attributes (like rename_all) as they aren't implemented yet
                Ok(())
            }
        })?;
    }

    Ok(ContainerAttrs { default })
}

/// Determine the name of a field, respecting a rename attribute.
pub fn name_of_field(field: &Field) -> Result<String> {
    let attrs = get(field)?;
    Ok(attrs.rename.unwrap_or_else(|| unraw(field.ident.as_ref().unwrap())))
}

/// Determine the name of a variant, respecting a rename attribute.
pub fn name_of_variant(var: &Variant) -> Result<String> {
    let mut rename = None;

    for attr in &var.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let s: LitStr = meta.value()?.parse()?;
                if rename.is_some() {
                    return Err(meta.error("duplicate rename attribute"));
                }
                rename = Some(s.value());
                Ok(())
            } else {
                Err(meta.error("unsupported attribute"))
            }
        })?;
    }

    Ok(rename.unwrap_or_else(|| unraw(&var.ident)))
}

fn unraw(ident: &Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_owned()
}