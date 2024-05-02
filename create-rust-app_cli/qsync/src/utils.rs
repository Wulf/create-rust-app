use inflector::Inflector;

/// Converts a `/home/user/path/to/file.rs` to ["home", "user", "path", "to", "file"]
/// Note: this trims the `.rs` extension from the filename (the last element in the vec)
pub fn file_path_to_vec_string(input_path: &std::path::Path) -> Vec<String> {
    let mut s: Vec<String> = vec![];

    for path in input_path.components() {
        let path_as_string = path.as_os_str().to_str().unwrap_or_default().to_string();
        let path_as_string = path_as_string.trim_end_matches(".rs").to_string();
        s.push(path_as_string.clone());
    }

    s.iter().map(Inflector::to_pascal_case).collect()
}

#[test]
fn test_file_path_to_vec_string() {
    let path = std::path::Path::new("/home/user/path/to/file.rs");
    let result = file_path_to_vec_string(path);
    assert_eq!(result, vec!["", "Home", "User", "Path", "To", "File"]);
}
