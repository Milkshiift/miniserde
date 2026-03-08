use proc_macro2::Ident;
use syn::{DeriveInput, Field, LitStr, Path, Result, Variant};

pub struct FieldAttrs {
    pub rename: Option<String>,
    pub aliases: Vec<String>,
    pub skip_serializing_if: Option<Path>,
    pub default: Default,
}

pub struct ContainerAttrs {
    pub default: Default,
    pub rename_all: Option<RenameRule>,
    pub tag: Option<String>,
}

#[derive(Clone)]
pub enum Default {
    None,
    Default,
    Path(Path),
}

#[derive(Clone, Copy)]
pub enum RenameRule {
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameRule {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "lowercase" => Some(Self::LowerCase),
            "UPPERCASE" => Some(Self::UpperCase),
            "PascalCase" => Some(Self::PascalCase),
            "camelCase" => Some(Self::CamelCase),
            "snake_case" => Some(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnakeCase),
            "kebab-case" => Some(Self::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(Self::ScreamingKebabCase),
            _ => None,
        }
    }

    fn apply_to_field(self, value: &str) -> String {
        let words = split_words(value);

        match self {
            Self::LowerCase => words.join(""),
            Self::UpperCase => words.join("").to_ascii_uppercase(),
            Self::PascalCase => words.iter().map(|word| capitalize(word)).collect(),
            Self::CamelCase => {
                let mut it = words.into_iter();
                let mut out = it.next().unwrap_or_default();
                for word in it {
                    out.push_str(&capitalize(&word));
                }
                out
            }
            Self::SnakeCase => words.join("_"),
            Self::ScreamingSnakeCase => words.join("_").to_ascii_uppercase(),
            Self::KebabCase => words.join("-"),
            Self::ScreamingKebabCase => words.join("-").to_ascii_uppercase(),
        }
    }
}

pub fn get(field: &Field) -> Result<FieldAttrs> {
    let mut rename = None;
    let mut aliases = Vec::new();
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
            } else if meta.path.is_ident("alias") {
                let s: LitStr = meta.value()?.parse()?;
                aliases.push(s.value());
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
        aliases,
        skip_serializing_if,
        default,
    })
}

pub fn get_container(input: &DeriveInput) -> Result<ContainerAttrs> {
    let mut default = Default::None;
    let mut rename_all = None;
    let mut tag = None;

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
            } else if meta.path.is_ident("rename_all") {
                let s: LitStr = meta.value()?.parse()?;
                if rename_all.is_some() {
                    return Err(meta.error("duplicate rename_all attribute"));
                }
                rename_all = Some(RenameRule::parse(&s.value()).ok_or_else(|| {
                    meta.error(format!("unsupported rename_all rule `{}`", s.value()))
                })?);
                Ok(())
            } else if meta.path.is_ident("tag") {
                let s: LitStr = meta.value()?.parse()?;
                if tag.is_some() {
                    return Err(meta.error("duplicate tag attribute"));
                }
                tag = Some(s.value());
                Ok(())
            } else {
                Ok(())
            }
        })?;
    }

    Ok(ContainerAttrs {
        default,
        rename_all,
        tag,
    })
}

pub fn name_of_field(field: &Field, container: &ContainerAttrs) -> Result<String> {
    Ok(deserialize_names_of_field(field, container)?
        .into_iter()
        .next()
        .unwrap())
}

pub fn deserialize_names_of_field(
    field: &Field,
    container: &ContainerAttrs,
) -> Result<Vec<String>> {
    let attrs = get(field)?;
    let ident = unraw(field.ident.as_ref().unwrap());

    let primary = match attrs.rename {
        Some(rename) => rename,
        None => match container.rename_all {
            Some(rule) => rule.apply_to_field(&ident),
            None => ident,
        },
    };

    let mut names = vec![primary];
    names.extend(attrs.aliases);
    Ok(names)
}

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

fn split_words(value: &str) -> Vec<String> {
    let words = value
        .split(|c: char| c == '_' || c == '-')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_lowercase())
        .collect::<Vec<_>>();

    if words.is_empty() {
        vec![value.to_ascii_lowercase()]
    } else {
        words
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => {
            let mut out = first.to_uppercase().collect::<String>();
            out.push_str(chars.as_str());
            out
        }
        None => String::new(),
    }
}
