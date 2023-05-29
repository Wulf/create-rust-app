use std::path::{Path, PathBuf};

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
    /// constant for the path to the project's frontend directory
    pub(crate) static ref FRONTEND_DIR: String = {
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
    };
    /// constant for the path to the project's manifest.json file
    pub(crate) static ref MANIFEST_PATH: String = {
        match std::env::var("CRA_MANIFEST_PATH") {
            Ok(dir) => dir,
            Err(_) => {
                #[cfg(not(feature = "plugin_workspace_support"))]
                {
                    "./frontend/dist/manifest.json".to_string()
                }
                #[cfg(feature = "plugin_workspace_support")]
                {
                    if *WORKSPACE_DIR == std::env::current_dir().unwrap() {
                        return "./frontend/dist/manifest.json".to_string();
                    } else {
                        return "../frontend/dist/manifest.json".to_string();
                    }
                }
            }
        }
    };
    /// constant for the path to the project's views directory
    pub(crate) static ref VIEWS_GLOB: String = {
        match std::env::var("CRA_VIEWS_GLOB") {
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
        }
    };


);
