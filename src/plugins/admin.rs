use crate::logger::file_msg;
use crate::project::add_dependency;
use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use rust_embed::RustEmbed;
use anyhow::Result;
use indoc::indoc;
use crate::fs;

pub struct Admin {}

#[derive(RustEmbed)]
#[folder = "template-plugin-admin"]
struct Asset;

impl Plugin for Admin {
  fn name(&self) -> &'static str {
    "Admin"
  }

  fn install(&self, install_config: InstallConfig) -> Result<()> {
    for filename in Asset::iter() {
      let file_contents = Asset::get(filename.as_ref()).unwrap();
      let mut file_path = std::path::PathBuf::from(&install_config.project_dir);
      file_path.push(".cargo/");
      file_path.push("admin/");
      file_path.push("dist/");
      file_path.push(filename.as_ref());
      let mut directory_path = std::path::PathBuf::from(&file_path);
      directory_path.pop();

      file_msg(filename.as_ref());
      std::fs::create_dir_all(directory_path)?;
      std::fs::write(file_path, file_contents)?;
    }
    
    /* Add dependencies */
    add_dependency(&install_config.project_dir, "diesel_migrations", toml::Value::String("1.4.0".into()))?;
    add_dependency(&install_config.project_dir, "diesel_migrations", toml::Value::String("{version=\"0.19.1\", features=[\"with-serde_json-1\"]}".into()))?;

    // TODO: Fix these appends/prepends by prepending the filepath with project_dir
    // currently, this works because we assume the current working directory is the project's root
    fs::replace(r#""concurrently": "^6.2.1""#, r#",
    "react-query": "^3.21.0""#);

    fs::replace("const App = () => {", r#"if (process.env.NODE_ENV === 'development') require('./setupDevelopment')
    
    const App = () => {"#);
    
    Ok(())
  }
}
