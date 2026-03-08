use crate::{attr, bound, fallback, private};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::BTreeMap;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Result,
};

pub fn derive(input: &DeriveInput) -> TokenStream {
    match try_expand(input) {
        Ok(expanded) => expanded,
        Err(error) => fallback::de(input, error),
    }
}

fn try_expand(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => derive_struct(input, fields),
        Data::Enum(enumeration) => derive_enum(input, enumeration),
        Data::Struct(_) => Err(Error::new(
            Span::call_site(),
            "currently only structs with named fields are supported",
        )),
        Data::Union(_) => Err(Error::new(
            Span::call_site(),
            "currently only structs and enums are supported by this derive",
        )),
    }
}

pub fn derive_struct(input: &DeriveInput, fields: &FieldsNamed) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let container_attrs = attr::get_container(input)?;

    let fieldname = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldty = fields.named.iter().map(|f| &f.ty);
    let fieldstr = fields
        .named
        .iter()
        .map(|f| attr::deserialize_names_of_field(f, &container_attrs))
        .collect::<Result<Vec<_>>>()?;

    let unwrap_logic = fields
        .named
        .iter()
        .map(|f| {
            let attrs = attr::get(f)?;
            let field_ident = &f.ident;

            match attrs.default {
                attr::Default::Path(path) => Ok(quote!(.unwrap_or_else(#path))),
                attr::Default::Default => Ok(quote!(.unwrap_or_default())),
                attr::Default::None => match &container_attrs.default {
                    attr::Default::Path(path) => {
                        Ok(quote!(.unwrap_or_else(|| #path().#field_ident)))
                    }
                    attr::Default::Default => {
                        Ok(quote!(.unwrap_or_else(|| <#ident #ty_generics as Default>::default().#field_ident)))
                    }
                    attr::Default::None => Ok(quote!(.take().ok_or(miniserde::Error)?)),
                },
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();
    let bound = parse_quote!(miniserde::Deserialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);
    let private2 = private;

    Ok(quote! {
        #[allow(deprecated, non_upper_case_globals)]
        const _: () = {
            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: miniserde::#private::Option<#ident #ty_generics>,
            }

            impl #impl_generics miniserde::Deserialize for #ident #ty_generics #bounded_where_clause {
                fn begin(__out: &mut miniserde::#private::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::#private::Option<Self>
                            as *mut __Visitor #ty_generics
                        }
                    }
                }
            }

            impl #impl_generics miniserde::de::Visitor for __Visitor #ty_generics #bounded_where_clause {
                fn map(&mut self) -> miniserde::Result<miniserde::#private::Box<dyn miniserde::de::Map + '_>> {
                    Ok(miniserde::#private::Box::new(__State {
                        #(
                            #fieldname: miniserde::Deserialize::default(),
                        )*
                        __out: &mut self.__out,
                    }))
                }
            }

            struct __State #wrapper_impl_generics #where_clause {
                #(
                    #fieldname: miniserde::#private2::Option<#fieldty>,
                )*
                __out: &'__a mut miniserde::#private::Option<#ident #ty_generics>,
            }

            impl #wrapper_impl_generics miniserde::de::Map for __State #wrapper_ty_generics #bounded_where_clause {
                fn key(&mut self, __k: &miniserde::#private::str) -> miniserde::Result<&mut dyn miniserde::de::Visitor> {
                    match __k {
                        #(
                            #( #fieldstr )|* => miniserde::#private2::Ok(miniserde::Deserialize::begin(&mut self.#fieldname)),
                        )*
                        _ => miniserde::#private::Ok(<dyn miniserde::de::Visitor>::ignore()),
                    }
                }

                fn finish(&mut self) -> miniserde::Result<()> {
                    #(
                        let #fieldname = self.#fieldname.take() #unwrap_logic;
                    )*
                    *self.__out = miniserde::#private::Some(#ident {
                        #(
                            #fieldname,
                        )*
                    });
                    miniserde::#private::Ok(())
                }
            }
        };
    })
}

pub fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> Result<TokenStream> {
    let container_attrs = attr::get_container(input)?;

    if let Some(tag) = container_attrs.tag {
        derive_internally_tagged_enum(input, enumeration, &tag)
    } else {
        derive_string_enum(input, enumeration)
    }
}

fn derive_string_enum(input: &DeriveInput, enumeration: &DataEnum) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Enums with generics are not supported",
        ));
    }

    let ident = &input.ident;

    let var_idents = enumeration
        .variants
        .iter()
        .map(|variant| match variant.fields {
            Fields::Unit => Ok(&variant.ident),
            _ => Err(Error::new_spanned(
                variant,
                "Invalid variant: only simple enum variants without fields are supported",
            )),
        })
        .collect::<Result<Vec<_>>>()?;
    let names = enumeration
        .variants
        .iter()
        .map(attr::name_of_variant)
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #[allow(deprecated, non_upper_case_globals)]
        const _: () = {
            #[repr(C)]
            struct __Visitor {
                __out: miniserde::#private::Option<#ident>,
            }

            impl miniserde::Deserialize for #ident {
                fn begin(__out: &mut miniserde::#private::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::#private::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }

            impl miniserde::de::Visitor for __Visitor {
                fn string(&mut self, s: &miniserde::#private::str) -> miniserde::Result<()> {
                    let value = match s {
                        #( #names => #ident::#var_idents, )*
                        _ => return miniserde::#private::Err(miniserde::Error),
                    };
                    self.__out = miniserde::#private::Some(value);
                    miniserde::#private::Ok(())
                }
            }
        };
    })
}

fn derive_internally_tagged_enum(
    input: &DeriveInput,
    enumeration: &DataEnum,
    tag: &str,
) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Enums with generics are not supported",
        ));
    }

    struct VariantField<'a> {
        ident: &'a syn::Ident,
        ty: &'a syn::Type,
        slot: usize,
        default: attr::Default,
    }

    struct VariantInfo<'a> {
        ident: &'a syn::Ident,
        name: String,
        fields: Vec<VariantField<'a>>,
        is_unit: bool,
    }

    let ident = &input.ident;
    let private2 = private;
    let mut slot_idents = Vec::<Ident>::new();
    let mut primary_to_slot = BTreeMap::<String, usize>::new();
    let mut key_to_slot = BTreeMap::<String, usize>::new();
    let mut variants = Vec::<VariantInfo<'_>>::new();

    for variant in &enumeration.variants {
        let variant_name = attr::name_of_variant(variant)?;

        match &variant.fields {
            Fields::Unit => variants.push(VariantInfo {
                ident: &variant.ident,
                name: variant_name,
                fields: Vec::new(),
                is_unit: true,
            }),
            Fields::Named(fields) => {
                let mut variant_fields = Vec::new();

                for field in &fields.named {
                    let field_ident = field.ident.as_ref().unwrap();
                    let field_attrs = attr::get(field)?;

                    let primary = field_attrs.rename.clone().unwrap_or_else(|| {
                        field_ident.to_string().trim_start_matches("r#").to_owned()
                    });

                    let slot = match primary_to_slot.get(&primary) {
                        Some(&slot) => slot,
                        None => {
                            let slot = slot_idents.len();
                            slot_idents
                                .push(Ident::new(&format!("__field{slot}"), Span::call_site()));
                            primary_to_slot.insert(primary.clone(), slot);
                            slot
                        }
                    };

                    let mut all_names = vec![primary];
                    all_names.extend(field_attrs.aliases);

                    for name in all_names {
                        if let Some(&existing) = key_to_slot.get(&name) {
                            if existing != slot {
                                return Err(Error::new_spanned(
                                    field,
                                    format!(
                                        "field name or alias `{name}` conflicts with another field"
                                    ),
                                ));
                            }
                        } else {
                            key_to_slot.insert(name, slot);
                        }
                    }

                    variant_fields.push(VariantField {
                        ident: field_ident,
                        ty: &field.ty,
                        slot,
                        default: field_attrs.default.clone(),
                    });
                }

                variants.push(VariantInfo {
                    ident: &variant.ident,
                    name: variant_name,
                    fields: variant_fields,
                    is_unit: false,
                });
            }
            _ => {
                return Err(Error::new_spanned(
                    variant,
                    "Invalid variant: internally tagged enums support only unit and named-field variants",
                ));
            }
        }
    }

    if key_to_slot.contains_key(tag) {
        return Err(Error::new_spanned(
            input,
            "tag name conflicts with an enum field name",
        ));
    }

    let tag_idents = variants
        .iter()
        .map(|variant| variant.ident)
        .collect::<Vec<_>>();
    let tag_names = variants
        .iter()
        .map(|variant| &variant.name)
        .collect::<Vec<_>>();
    let key_names = key_to_slot.keys().collect::<Vec<_>>();
    let key_slots = key_to_slot
        .values()
        .map(|&index| &slot_idents[index])
        .collect::<Vec<_>>();

    let construct_arms = variants.iter().map(|variant| {
        let variant_ident = variant.ident;

        if variant.is_unit {
            quote! {
                __Tag::#variant_ident => {
                    *self.__out = miniserde::#private2::Some(#ident::#variant_ident);
                }
            }
        } else {
            let field_reads = variant.fields.iter().map(|field| {
                let field_ident = field.ident;
                let field_ty = field.ty;
                let slot_ident = &slot_idents[field.slot];
                let default_expr = enum_default_expr(&field.default, field_ty);

                quote! {
                    let #field_ident: #field_ty = match self.#slot_ident.take() {
                        miniserde::#private2::Some(__value) => {
                            miniserde::json::from_value(__value).map_err(|_| miniserde::Error)?
                        }
                        miniserde::#private2::None => #default_expr,
                    };
                }
            });

            let field_idents = variant.fields.iter().map(|field| field.ident);

            quote! {
                __Tag::#variant_ident => {
                    #(
                        #field_reads
                    )*
                    *self.__out = miniserde::#private2::Some(#ident::#variant_ident {
                        #(
                            #field_idents,
                        )*
                    });
                }
            }
        }
    });

    Ok(quote! {
        #[allow(deprecated, non_upper_case_globals)]
        const _: () = {
            #[repr(C)]
            struct __Visitor {
                __out: miniserde::#private2::Option<#ident>,
            }

            impl miniserde::Deserialize for #ident {
                fn begin(__out: &mut miniserde::#private2::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::#private2::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }

            impl miniserde::de::Visitor for __Visitor {
                fn map(&mut self) -> miniserde::Result<miniserde::#private2::Box<dyn miniserde::de::Map + '_>> {
                    Ok(miniserde::#private2::Box::new(__State {
                        #(
                            #slot_idents: miniserde::Deserialize::default(),
                        )*
                        __tag: miniserde::Deserialize::default(),
                        __out: &mut self.__out,
                    }))
                }
            }

            enum __Tag {
                #(
                    #tag_idents,
                )*
            }

            #[repr(C)]
            struct __TagVisitor {
                __out: miniserde::#private2::Option<__Tag>,
            }

            impl miniserde::Deserialize for __Tag {
                fn begin(__out: &mut miniserde::#private2::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::#private2::Option<Self>
                            as *mut __TagVisitor
                        }
                    }
                }
            }

            impl miniserde::de::Visitor for __TagVisitor {
                fn string(&mut self, s: &miniserde::#private2::str) -> miniserde::Result<()> {
                    let value = match s {
                        #( #tag_names => __Tag::#tag_idents, )*
                        _ => return miniserde::#private2::Err(miniserde::Error),
                    };
                    self.__out = miniserde::#private2::Some(value);
                    miniserde::#private2::Ok(())
                }
            }

            struct __State<'__a> {
                #(
                    #slot_idents: miniserde::#private2::Option<miniserde::json::Value>,
                )*
                __tag: miniserde::#private2::Option<__Tag>,
                __out: &'__a mut miniserde::#private2::Option<#ident>,
            }

            impl<'__a> miniserde::de::Map for __State<'__a> {
                fn key(&mut self, __k: &miniserde::#private2::str) -> miniserde::Result<&mut dyn miniserde::de::Visitor> {
                    match __k {
                        #tag => miniserde::#private2::Ok(miniserde::Deserialize::begin(&mut self.__tag)),
                        #(
                            #key_names => miniserde::#private2::Ok(miniserde::Deserialize::begin(&mut self.#key_slots)),
                        )*
                        _ => miniserde::#private2::Ok(<dyn miniserde::de::Visitor>::ignore()),
                    }
                }

                fn finish(&mut self) -> miniserde::Result<()> {
                    let __tag = self.__tag.take().ok_or(miniserde::Error)?;

                    match __tag {
                        #(
                            #construct_arms
                        )*
                    }

                    miniserde::#private2::Ok(())
                }
            }
        };
    })
}

fn enum_default_expr(default: &attr::Default, field_ty: &syn::Type) -> TokenStream {
    match default {
        attr::Default::Path(path) => quote!(#path()),
        attr::Default::Default => quote!(<#field_ty as Default>::default()),
        attr::Default::None => quote!(return Err(miniserde::Error)),
    }
}
