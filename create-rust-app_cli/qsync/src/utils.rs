use inflector::Inflector;

/// Converts a `/home/user/path/to/file.rs` to ["home", "user", "path", "to", "file"]
/// Note: this trims the `.rs` extension from the filename (the last element in the vec)
pub fn file_path_to_vec_string(input_path: &std::path::Path) -> Vec<String> {
    let mut s: Vec<String> = vec![];

    let mut copy = false;
    for path in input_path.components() {
        let path_as_string = path.as_os_str().to_str().unwrap_or_default().to_string();
        let path_as_string = path_as_string.trim_end_matches(".rs").to_string();
        if copy {
            s.push(path_as_string.clone());
        }
        if path_as_string.eq_ignore_ascii_case("services") {
            copy = true;
        }
    }

    s.iter().map(|s| s.to_pascal_case()).collect()
}
