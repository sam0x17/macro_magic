//! This crate contains most of the internal implementation of the macros in the
//! `macro_magic_macros` crate. For the most part, the proc macros in `macro_magic_macros` just
//! call their respective `_internal` variants in this crate.

use convert_case::{Case, Casing};
use derive_syn_parse::Parse;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::FnArg;
use syn::Pat;
use syn::{
    parse::Nothing, parse2, parse_quote, token::Comma, Error, Ident, Item, ItemFn, Path, Result,
    Token, Visibility,
};

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(proc_macro);
    custom_keyword!(proc_macro_attribute);
    custom_keyword!(proc_macro_derive);
}

pub struct MiscTokens {
    pub tokens: TokenStream2,
}

impl syn::parse::Parse for MiscTokens {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(MiscTokens {
            tokens: input.to_string().to_token_stream(),
        })
    }
}

#[derive(Parse)]
pub struct ForwardTokensArgs {
    pub source: Path,
    _comma: Comma,
    pub target: Path,
}

#[derive(Parse)]
pub struct ForwardedTokens {
    pub target_path: Path,
    _comma: Comma,
    pub item: Item,
}

/// Used to parse the args for the [`import_tokens_internal`] function.
#[derive(Parse)]
pub struct ImportTokensArgs {
    _let: Token![let],
    pub tokens_var_ident: Ident,
    _eq: Token![=],
    pub source_path: Path,
}

#[derive(Parse)]
pub struct ImportedTokens {
    pub tokens_var_ident: Ident,
    _comma: Comma,
    pub item: Item,
}

/// Appends `member` to the end of the `::macro_magic::__private` path and returns the
/// resulting [`Path`]
pub fn private_path(member: &TokenStream2) -> Path {
    parse_quote!(::macro_magic::__private::#member)
}

/// "Flattens" an ident by converting it to snake case. This is used by
/// [`export_tokens_macro_ident`].
pub fn flatten_ident(ident: &Ident) -> Ident {
    Ident::new(
        ident.to_string().to_case(Case::Snake).as_str(),
        ident.span(),
    )
}

/// Produces the full path for the auto-generated callback-based tt macro that allows us to
/// forward tokens across crate boundaries
pub fn export_tokens_macro_ident(ident: &Ident) -> Ident {
    let ident = flatten_ident(&ident);
    let ident_string = format!("__export_tokens_tt_{}", ident.to_token_stream().to_string());
    Ident::new(ident_string.as_str(), Span::call_site())
}

/// The internal code behind the `#[export_tokens]` attribute macro. The `attr` variable
/// contains the tokens for the optional naming [`Ident`] (necessary on [`Item`]s that don't
/// have an inherent [`Ident`]) is the optional `attr` and the `tokens` variable is the tokens
/// for the [`Item`] the attribute macro can be attached to. The `attr` variable can be blank
/// tokens for supported items, which includes every valid [`syn::Item`] except for
/// [`syn::ItemForeignMod`], [`syn::ItemUse`], [`syn::ItemImpl`], and [`Item::Verbatim`], which
/// all require `attr` to be specified.
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
    let ident = export_tokens_macro_ident(&ident);
    Ok(quote! {
        #[macro_export]
        macro_rules! #ident {
            ($tokens_var:path, $callback:path) => {
                $callback! {
                    $tokens_var,
                    #item
                }
            };
        }
        #[allow(unused)]
        #item
    })
}

/// The internal implementation for the `import_tokens` macro. You can call this in your own
/// proc macros to make use of the `import_tokens` functionality directly. The arguments should
/// be a [`TokenStream2`] that can parse into an [`ImportTokensArgs`] successfully. That is a
/// valid `let` variable declaration set to equal a path where an `#[export_tokens]` with the
/// specified ident can be found.
///
/// ### Example:
/// ```
/// use macro_magic_core::*;
/// use quote::quote;
///
/// let some_ident = quote!(my_tokens);
/// let some_path = quote!(other_crate::exported_item);
/// let tokens = import_tokens_internal(quote!(let #some_ident = other_crate::ExportedItem)).unwrap();
/// assert_eq!(
///     tokens.to_string(),
///     "other_crate :: __export_tokens_tt_exported_item ! { my_tokens , \
///     :: macro_magic :: __private :: import_tokens_inner }");
/// ```
/// If these tokens were emitted as part of a proc macro, they would expand to a variable
/// declaration like:
/// ```ignore
/// let my_tokens: TokenStream2;
/// ```
/// where `my_tokens` contains the tokens of `ExportedItem`.
pub fn import_tokens_internal<T: Into<TokenStream2>>(tokens: T) -> Result<TokenStream2> {
    let args = parse2::<ImportTokensArgs>(tokens.into())?;
    let Some(source_ident_seg) = args.source_path.segments.last() else { unreachable!("must have at least one segment") };
    let source_ident_seg = export_tokens_macro_ident(&source_ident_seg.ident);
    let source_path = if args.source_path.segments.len() > 1 {
        let Some(crate_seg) = args.source_path.segments.first() else {
            unreachable!("path has at least two segments, so there is a first segment");
        };
        quote!(#crate_seg::#source_ident_seg)
    } else {
        quote!(#source_ident_seg)
    };
    let inner_macro_path = private_path(&quote!(import_tokens_inner));
    let tokens_var_ident = args.tokens_var_ident;
    Ok(quote! {
        #source_path! { #tokens_var_ident, #inner_macro_path }
    })
}

/// The internal implementation for the `import_tokens_inner` macro. You shouldn't need to
/// call this in any circumstances but it is provided just in case.
pub fn import_tokens_inner_internal<T: Into<TokenStream2>>(tokens: T) -> Result<TokenStream2> {
    let parsed = parse2::<ImportedTokens>(tokens.into())?;
    let tokens_string = parsed.item.to_token_stream().to_string();
    let ident = parsed.tokens_var_ident;
    let token_stream_2 = private_path(&quote!(TokenStream2));
    Ok(quote! {
        let #ident = #tokens_string.parse::<#token_stream_2>().expect("failed to parse quoted tokens");
    })
}

pub fn forward_tokens_internal<T: Into<TokenStream2>>(tokens: T) -> Result<TokenStream2> {
    let args = parse2::<ForwardTokensArgs>(tokens.into())?;
    let Some(source_ident_seg) = args.source.segments.last() else { unreachable!("must have at least one segment") };
    let source_ident_seg = export_tokens_macro_ident(&source_ident_seg.ident);
    let source_path = if args.source.segments.len() > 1 {
        let Some(crate_seg) = args.source.segments.first() else {
            unreachable!("path has at least two segments, so there is a first segment");
        };
        quote!(#crate_seg::#source_ident_seg)
    } else {
        quote!(#source_ident_seg)
    };
    let inner_macro_path = private_path(&quote!(forward_tokens_inner));
    let target_path = args.target;
    Ok(quote! {
        #source_path! { #target_path, #inner_macro_path }
    })
}

pub fn forward_tokens_inner_internal<T: Into<TokenStream2>>(tokens: T) -> Result<TokenStream2> {
    let parsed = parse2::<ForwardedTokens>(tokens.into())?;
    let target_path = parsed.target_path;
    let imported_tokens = parsed.item;
    Ok(quote! {
        #target_path! {
            #imported_tokens
        }
    })
}

pub fn import_tokens_attr_outer_quote<T: Into<TokenStream2>>(
    path_tokens: T,
    inner_macro_ident_tokens: T,
) -> TokenStream2 {
    let path = match syn::parse2::<syn::Path>(path_tokens.into()) {
        Ok(path) => path,
        Err(e) => return e.to_compile_error().into(),
    };
    let inner_macro_ident = match syn::parse2::<syn::Ident>(inner_macro_ident_tokens.into()) {
        Ok(ident) => ident,
        Err(e) => return e.to_compile_error().into(),
    };
    quote::quote! {
        ::macro_magic::forward_tokens!(#inner_macro_ident, #path)
    }
}

pub fn import_tokens_attr_internal<T1: Into<TokenStream2>, T2: Into<TokenStream2>>(
    attr: T1,
    tokens: T2,
) -> Result<TokenStream2> {
    parse2::<Nothing>(attr.into())?;
    let proc_fn = parse2::<ItemFn>(tokens.into())?;
    let Visibility::Public(_) = proc_fn.vis else { return Err(Error::new(proc_fn.vis.span(), "Visibility must be public")) };
    if proc_fn
        .attrs
        .iter()
        .find(|attr| {
            syn::parse2::<keywords::proc_macro_attribute>(attr.path.to_token_stream()).is_ok()
        })
        .is_none()
    {
        return Err(Error::new(
            proc_fn.sig.ident.span(),
            "can only be attached to a function with #[proc_macro_attribute]",
        ));
    };

    // parsing complete, we have a valid attribute macro function (all other errors will be
    // handled by the presence of the #[proc_macro_atribute] attribute)

    // outer macro
    let orig_sig = proc_fn.sig;
    let orig_stmts = proc_fn.block.stmts;
    let orig_attrs = proc_fn.attrs;

    // inner macro
    let inner_macro_ident = format_ident!("__import_tokens_attr_{}_inner", orig_sig.ident);
    let mut inner_sig = orig_sig.clone();
    inner_sig.ident = inner_macro_ident.clone();

    // source path
    let Some(FnArg::Typed(first_arg)) = orig_sig.inputs.first() else {
        unreachable!("missing first arg");
    };
    let Pat::Ident(first_arg_ident) = *first_arg.pat.clone() else {
        panic!("invalid first arg");
    };

    // attached item tokens
    let Some(FnArg::Typed(second_arg)) = orig_sig.inputs.last() else {
        unreachable!("missing second arg");
    };
    let Pat::Ident(second_arg_ident) = *second_arg.pat.clone() else {
        panic!("invalid second arg");
    };
    println!("first_arg_ident: {}", first_arg_ident.ident.to_string());
    println!("inner_macro_ident: {}", inner_macro_ident.to_string());

    // final quoted tokens
    Ok(quote! {
        #(#orig_attrs)
        *
        pub #orig_sig {
            use ::macro_magic::__private::*;
            let attached_item = syn::parse_macro_input!(#second_arg_ident as syn::Item);
            println!("outer macro input: {}::TokenStream", #first_arg_ident.to_string());
            let stuff = ::macro_magic::core::import_tokens_attr_outer_quote(
                #first_arg_ident.into(),
                quote::quote!(#inner_macro_ident)
            );
            println!("outer macro output: {}", stuff.to_string());
            stuff.into()
        }

        #[doc(hidden)]
        #[proc_macro]
        pub #inner_sig {
            println!("inner_macro input: {}", #first_arg_ident.to_string());
            #(#orig_stmts)
            *
        }
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
        .contains("__export_tokens_tt_my_struct"));
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

    #[test]
    fn import_tokens_internal_simple_path() {
        assert!(
            import_tokens_internal(quote!(let tokens = my_crate::SomethingCool))
                .unwrap()
                .to_string()
                .contains("__export_tokens_tt_something_cool")
        );
    }

    #[test]
    fn import_tokens_internal_flatten_long_paths() {
        assert!(import_tokens_internal(
            quote!(let tokens = my_crate::some_mod::complex::SomethingElse)
        )
        .unwrap()
        .to_string()
        .contains("__export_tokens_tt_something_else"));
    }

    #[test]
    fn import_tokens_internal_invalid_token_ident() {
        assert!(import_tokens_internal(quote!(let 3 * 2 = my_crate::something)).is_err());
    }

    #[test]
    fn import_tokens_internal_invalid_path() {
        assert!(import_tokens_internal(quote!(let my_tokens = 2 - 2)).is_err());
    }

    #[test]
    fn import_tokens_inner_internal_basic() {
        assert!(import_tokens_inner_internal(quote! {
            my_ident,
            fn my_function() -> u32 {
                33
            }
        })
        .unwrap()
        .to_string()
        .contains("my_ident"));
    }

    #[test]
    fn import_tokens_inner_internal_impl() {
        assert!(import_tokens_inner_internal(quote! {
            another_ident,
            impl Something for MyThing {
                fn something() -> CoolStuff {
                    CoolStuff {}
                }
            }
        })
        .unwrap()
        .to_string()
        .contains("something ()"));
    }

    #[test]
    fn import_tokens_inner_internal_missing_comma() {
        assert!(import_tokens_inner_internal(quote! {
            {
                another_ident
                impl Something for MyThing {
                    fn something() -> CoolStuff {
                        CoolStuff {}
                    }
                }
            }
        })
        .is_err());
    }

    #[test]
    fn import_tokens_inner_internal_non_item() {
        assert!(import_tokens_inner_internal(quote! {
            {
                another_ident,
                2 + 2
            }
        })
        .is_err());
    }
}
