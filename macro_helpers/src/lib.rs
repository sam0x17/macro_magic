use quote::{ToTokens, __private::Span};
use syn::{spanned::Spanned, Error, Ident, Path, TypePath};

pub fn get_const_name(name: String) -> String {
    format!("__EXPORT_TOKENS__{}", name.replace(" ", "").to_uppercase())
}

pub fn get_const_path(path: &TypePath) -> Result<Path, Error> {
    let mut path = path.path.clone();
    let Some(mut last) = path.segments.last_mut() else {
        return Err(Error::new(path.span(), "Empty paths cannot be expanded!"))
    };
    last.ident = Ident::new(
        get_const_name(last.to_token_stream().to_string()).as_str(),
        Span::call_site().into(),
    );
    Ok(path)
}
