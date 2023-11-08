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
        match std::env::var("CRA_FRONTEND_DIR") {
            Ok(dir) => dir,
            Err(_) => {
                #[cfg(not(feature = "plugin_workspace_support"))]
                {
                    "./frontend".to_string()
                }
                #[cfg(feature = "plugin_workspace_support")]
                {
                    if *WORKSPACE_DIR == std::env::current_dir().unwrap() {
                        // this is for when cargo run is run from the workspace root
                        return "./frontend".to_string();
                    } else {
                        // this is for when cargo run is run from teh backend directory
                        return "../frontend".to_string();
                    }
                }
            }
        }
    })
}
/// fn for the path to the project's manifest.json file
pub(crate) fn manifest_path() -> &'static str {
    static MANIFEST_PATH: OnceLock<String> = OnceLock::new();
    MANIFEST_PATH.get_or_init(|| match std::env::var("CRA_MANIFEST_PATH") {
        Ok(dir) => dir,
        Err(_) => {
            #[cfg(not(feature = "plugin_workspace_support"))]
            {
                "./frontend/dist/manifest.json".to_string()
            }
            #[cfg(feature = "plugin_workspace_support")]
            {
                if *WORKSPACE_DIR == std::env::current_dir().unwrap() {
                    "./frontend/dist/manifest.json".to_string();
                } else {
                    "../frontend/dist/manifest.json".to_string();
                }
            }
        }
    })
}
/// fn for the path to the project's views directory
pub(crate) fn views_glob() -> &'static str {
    static VIEWS_GLOB: OnceLock<String> = OnceLock::new();
    VIEWS_GLOB.get_or_init(|| match std::env::var("CRA_VIEWS_GLOB") {
        Ok(dir) => dir,
        Err(_) => {
            #[cfg(not(feature = "plugin_workspace_support"))]
            {
                "backend/views/**/*.html".to_string()
            }
            #[cfg(feature = "plugin_workspace_support")]
            {
                if *WORKSPACE_DIR == std::env::current_dir().unwrap() {
                    return "backend/views/**/*.html".to_string();
                } else {
                    return "views/**/*.html".to_string();
                }
            }
        }
    })
}
