pub fn get_const_name(name: String) -> String {
    format!("__EXPORT_TOKENS__{}", name.replace(" ", "").to_uppercase())
}
