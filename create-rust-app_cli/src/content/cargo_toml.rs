use toml::Value::Table;
use crate::logger;

pub fn add_dependency(
    project_dir: &std::path::PathBuf,
    log_name: &str,
    dependency_entry: &str,
) -> Result<(), std::io::Error> {
    let mut path = std::path::PathBuf::from(project_dir);
    path.push("Cargo.toml");

    let toml: String = std::fs::read_to_string(&path).unwrap();

    let parsed_toml = toml.parse::<toml::Value>();

    if parsed_toml.is_err() {
        println!("{:#?}", toml);
        panic!("Fatal/Invalid state: couldn't add dependency due to Cargo.toml parsing error.");
    }

    let mut parsed_toml = parsed_toml.unwrap();

    let root: &mut toml::value::Table = parsed_toml.as_table_mut().unwrap();

    let deps_table: &mut toml::value::Table = root
        .get_mut("dependencies")
        .unwrap()
        .as_table_mut()
        .unwrap();
    deps_table.insert("replace_me".to_string(), toml::Value::String("123".to_string()));

    let updated_toml = toml::to_string(&parsed_toml).unwrap();
    let updated_toml = updated_toml.replace("replace_me = \"123\"", dependency_entry);

    logger::dependency_msg(log_name);

    std::fs::write(&path, updated_toml)?;

    Ok(())
}

