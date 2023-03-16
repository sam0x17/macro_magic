use convert_case::{Case, Casing};
use derive_syn_parse::Parse;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse2;
use syn::{
    parse::Nothing,
    parse_macro_input,
    token::{Brace, Comma},
    Ident, Item, Path, Result, Token,
};

#[derive(Parse)]
pub struct ImportTokensArgs {
    _let: Token![let],
    tokens_var_ident: Ident,
    _eq: Token![=],
    source_path: Path,
}

#[derive(Parse)]
pub struct ImportedTokensBraceContents {
    tokens_var_ident: Ident,
    _comma: Comma,
    item: Item,
}

#[derive(Parse)]
pub struct ImportedTokensBrace {
    #[brace]
    _braces: Brace,
    #[inside(_braces)]
    contents: ImportedTokensBraceContents,
}

pub fn flatten_ident(ident: &Ident) -> Ident {
    Ident::new(
        ident.to_string().to_case(Case::Snake).as_str(),
        ident.span(),
    )
}

pub fn export_tokens_macro_ident(ident: &Ident) -> Ident {
    let ident = flatten_ident(&ident);
    let ident_string = format!("__export_tokens_tt_{}", ident.to_token_stream().to_string());
    Ident::new(ident_string.as_str(), Span::call_site())
}

pub fn export_tokens_internal<T: Into<TokenStream2>, E: Into<TokenStream2>>(
    attr: T,
    tokens: E,
) -> Result<TokenStream2> {
    let attr = attr.into();
    let item: Item = parse2(tokens.into())?;
    let ident = match item.clone() {
        Item::Const(item_const) => Some(item_const.ident),
        Item::Enum(item_enum) => Some(item_enum.ident),
        Item::ExternCrate(item_extern_crate) => Some(item_extern_crate.ident),
        Item::Fn(item_fn) => Some(item_fn.sig.ident),
        Item::Macro(item_macro) => item_macro.ident, // note this one might not have an Ident as well
        Item::Macro2(item_macro2) => Some(item_macro2.ident),
        Item::Mod(item_mod) => Some(item_mod.ident),
        Item::Static(item_static) => Some(item_static.ident),
        Item::Struct(item_struct) => Some(item_struct.ident),
        Item::Trait(item_trait) => Some(item_trait.ident),
        Item::TraitAlias(item_trait_alias) => Some(item_trait_alias.ident),
        Item::Type(item_type) => Some(item_type.ident),
        Item::Union(item_union) => Some(item_union.ident),
        // Item::ForeignMod(item_foreign_mod) => None,
        // Item::Use(item_use) => None,
        // Item::Impl(item_impl) => None,
        _ => None,
    };
    let ident = match ident {
        Some(ident) => {
            if let Ok(_) = parse2::<Nothing>(attr.clone()) {
                ident
            } else {
                parse2::<Ident>(attr)?
            }
        }
        None => parse2::<Ident>(attr)?,
    };
    let ident = flatten_ident(&ident);
    Ok(quote! {
        #[macro_export]
        macro_rules! #ident {
            ($tokens_var:ident, $callback:path) => {
                $callback! {
                    {
                        $tokens_var,
                        #item
                    }
                }
            };
        }
        #[allow(unused)]
        #item
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_tokens_internal_missing_ident() {
        assert!(export_tokens_internal(quote!(), quote!(impl MyTrait for Something)).is_err());
    }

    #[test]
    fn export_tokens_internal_normal_no_ident() {
        assert!(export_tokens_internal(
            quote!(),
            quote!(
                struct MyStruct {}
            )
        )
        .unwrap()
        .to_string()
        .contains("my_struct"));
    }

    #[test]
    fn export_tokens_internal_normal_ident() {
        assert!(export_tokens_internal(
            quote!(some_name),
            quote!(
                struct Something {}
            ),
        )
        .unwrap()
        .to_string()
        .contains("some_name"));
    }

    #[test]
    fn export_tokens_internal_generics_no_ident() {
        assert!(export_tokens_internal(
            quote!(),
            quote!(
                struct MyStruct<T> {}
            ),
        )
        .unwrap()
        .to_string()
        .contains("my_struct {"));
    }

    #[test]
    fn export_tokens_internal_bad_ident() {
        assert!(export_tokens_internal(
            quote!(Something<T>),
            quote!(
                struct MyStruct {}
            ),
        )
        .is_err());
        assert!(export_tokens_internal(
            quote!(some::path),
            quote!(
                struct MyStruct {}
            ),
        )
        .is_err());
    }
}
