use derive_syn_parse::Parse;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse2;
use syn::{
    parse::Nothing,
    parse_macro_input,
    token::{Brace, Comma},
    FnArg, Ident, Item, ItemFn, Pat, Path, Result, Token, Visibility,
};

pub fn export_tokens_macro_ident(ident: &Ident) -> Ident {
    let ident_string = format!("__export_tokens_tt_{}", ident.to_token_stream().to_string());
    Ident::new(ident_string.as_str(), Span::call_site())
}

pub fn export_tokens_internal<T: Into<TokenStream2>, E: Into<TokenStream2>>(
    tokens: T,
    attr: E,
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
            parse2::<Nothing>(attr)?;
            ident
        }
        None => parse2::<Ident>(attr)?,
    };
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
