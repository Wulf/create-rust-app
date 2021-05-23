#[allow(dead_code)]
fn try_var_or(key: &str, default: &str) -> Result<String, std::env::VarError> {
    return match std::env::var(key) {
        Ok(s) => Ok(s),
        Err(_) => Ok(default.to_string()),
    };
}

#[allow(dead_code)]
pub fn var(key: &str, default: &str) -> String {
    return try_var_or(key, default).unwrap();
}
