use crate::utils::fs;
use crate::utils::logger::file_msg;
use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use anyhow::Result;
use rust_embed::RustEmbed;

pub struct Dev {}

#[derive(RustEmbed)]
#[folder = "template-plugin-dev"]
struct Asset;

impl Plugin for Dev {
    fn name(&self) -> &'static str {
        "Dev"
    }

    fn install(&self, install_config: InstallConfig) -> Result<()> {
        for filename in Asset::iter() {
            if filename.contains(".cargo/admin") && !filename.contains(".cargo/admin/dist") {
                continue;
            }

            let file_contents = Asset::get(filename.as_ref()).unwrap();
            let mut file_path = std::path::PathBuf::from(&install_config.project_dir);
            file_path.push(filename.as_ref());
            let mut directory_path = std::path::PathBuf::from(&file_path);
            directory_path.pop();

            file_msg(filename.as_ref());
            std::fs::create_dir_all(directory_path)?;
            std::fs::write(file_path, file_contents)?;
        }

        // TODO: Fix these appends/prepends by prepending the filepath with project_dir
        // currently, this works because we assume the current working directory is the project's root

        // TODO: don't use concurrently as the anchor for new frontend dependencies
        fs::replace(
            "frontend/package.json",
            r#""concurrently": "^6.2.1""#,
            r#""concurrently": "^6.2.1",
    "react-query": "^3.21.0""#,
        )?;

        fs::replace(
            "frontend/src/App.tsx",
            "const App = () => {",
            r#"if (process.env.NODE_ENV === 'development') require('./setupDevelopment')
    
    const App = () => {"#,
        )?;

        fs::replace(
            "backend/main.rs",
            "let mut app = Route::new().nest(\"/api\", api);",
            r#"#[cfg(debug_assertions)]
    {
        api = api.nest("/development", create_rust_app::dev::api());
    }

    let mut app = Route::new().nest("/api", api);

    #[cfg(debug_assertions)]
    {
        app = app.at(
            "*",
            StaticFilesEndpoint::new(".cargo/admin/dist").index_file("admin.html"),
        );
    }"#
        )?;

        Ok(())
    }
}
