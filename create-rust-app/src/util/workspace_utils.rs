use std::path::{Path, PathBuf};
use std::sync::OnceLock; // use LazyLock instead once that's stable

use lazy_static::lazy_static;

lazy_static!(
    pub static ref WORKSPACE_DIR: PathBuf = {
        let output = std::process::Command::new(env!("CARGO"))
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .unwrap()
            .stdout;
        let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
        cargo_path.parent().unwrap().to_path_buf()
        // .to_str()
        // .unwrap()
        // .to_owned()
    };
);

/// fn for the path to the project's frontend directory
pub(crate) fn frontend_dir() -> &'static str {
    static FRONTEND_DIR: OnceLock<String> = OnceLock::new();
    FRONTEND_DIR.get_or_init(|| {
        std::env::var("CRA_FRONTEND_DIR").unwrap_or_else(|_| {
            match (
                cfg!(feature = "plugin_workspace_support"), // this can be replaced by a function that tells us if we're using workspaces, instead of using a feature flag
                std::env::current_dir(),
            ) {
                // this is for when cargo run is run from the backend directory
                (true, Ok(dir)) if dir != *WORKSPACE_DIR => "../frontend".to_string(),
                // default: when cargo run is run from the workspace root, or we couldn't get the current directory, or we aren't using workspaces
                _ => "./frontend".to_string(),
            }
        })
    })
}
/// fn for the path to the project's manifest.json file
pub(crate) fn manifest_path() -> &'static str {
    static MANIFEST_PATH: OnceLock<String> = OnceLock::new();
    MANIFEST_PATH.get_or_init(|| {
        std::env::var("CRA_FRONTEND_DIR").unwrap_or_else(|_| {
            match (
                cfg!(feature = "plugin_workspace_support"), // this can be replaced by a function that tells us if we're using workspaces, instead of using a feature flag
                std::env::current_dir(),
            ) {
                (true, Ok(dir)) if dir != *WORKSPACE_DIR => {
                    "../frontend/dist/manifest.json".to_string()
                }
                _ => "./frontend/dist/manifest.json".to_string(),
            }
        })
    })
}
/// fn for the path to the project's views directory
pub(crate) fn views_glob() -> &'static str {
    static VIEWS_GLOB: OnceLock<String> = OnceLock::new();
    VIEWS_GLOB.get_or_init(|| {
        std::env::var("CRA_VIEWS_GLOB").unwrap_or_else(|_| {
            match (
                cfg!(feature = "plugin_workspace_support"), // this can be replaced by a function that tells us if we're using workspaces, instead of using a feature flag
                std::env::current_dir(),
            ) {
                (true, Ok(dir)) if dir != *WORKSPACE_DIR => "views/**/*.html".to_string(),
                _ => "backend/views/**/*.html".to_string(),
            }
        })
    })
}
