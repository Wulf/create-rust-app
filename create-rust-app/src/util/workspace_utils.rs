use std::path::Path;
use std::sync::OnceLock; // use LazyLock instead once that's stable

/// OnceLock wrapper around output of `cargo locate-project --workspace --message-format=plain`
///
/// if the command fails (e.g. if we're in a container or otherwise don't have access to the projects source code), or output can't be parsed,  we return None
fn cargo_locate_project_workspace() -> Option<&'static str> {
    static CARGO_LP_WORKSPACE: OnceLock<Option<String>> = OnceLock::new();
    CARGO_LP_WORKSPACE
        .get_or_init(|| {
            let output = std::process::Command::new(env!("CARGO"))
                .arg("locate-project")
                .arg("--workspace")
                .arg("--message-format=plain")
                .output()
                .ok()?;
            let cargo_path = Path::new(std::str::from_utf8(&output.stdout).ok()?.trim());
            Some(
                cargo_path
                    .parent()
                    .and_then(|p| p.canonicalize().ok().or(Some(p.to_path_buf())))? // if we can't canonicalize the path, just use the original path
                    .to_str()?
                    .to_owned(),
            )
        })
        .as_deref()
}

/// OnceLock wrapper around output of `cargo locate-project --message-format=plain`
///
/// if the command fails (e.g. if we're in a container or otherwise don't have access to the projects source code), or output can't be parsed, we return None
fn cargo_locate_project() -> Option<&'static str> {
    static CARGO_LP: OnceLock<Option<String>> = OnceLock::new();
    CARGO_LP
        .get_or_init(|| {
            let output = std::process::Command::new(env!("CARGO"))
                .arg("locate-project")
                .arg("--workspace")
                .arg("--message-format=plain")
                .output()
                .ok()?;
            let cargo_toml_path = Path::new(std::str::from_utf8(&output.stdout).ok()?.trim())
                .parent()
                .and_then(|p| p.canonicalize().ok().or(Some(p.to_path_buf())))? // if we can't canonicalize the path, just use the original path, paths should be canonicalized anyway but this is just to guarentee it
                .to_str()?
                .to_owned();
            Some(cargo_toml_path)
        })
        .as_deref()
}

/// fn for the path to the project's frontend directory
pub(crate) fn frontend_dir() -> &'static str {
    static FRONTEND_DIR: OnceLock<String> = OnceLock::new();
    FRONTEND_DIR.get_or_init(|| {
        std::env::var("CRA_FRONTEND_DIR").unwrap_or_else(|_| {
            match (
                cargo_locate_project_workspace(),
                cargo_locate_project(),
                env!("CARGO_MANIFEST_DIR"),
            ) {
                // if we're in a container or something, both functions will fail and return None, so this case won't be hit
                // if we aren't using workspaces, both functions will return the same value, so this case won't be hit
                // if we are using workspaces, and running in the workspace root, both functions will succeed and return the same value, so this case won't be hit
                // if we're using workspaces, but executing from, say, the backend directory, the functions will return different values, so this case **will** be hit
                // if the executable is being run from some other location, the functions might fail, they won't if the executable is being run in a directory containing another cargo project
                //      - but in that case, the "CARGO_MANIFEST_DIR" env var will not be the same as the output of the second function, so this case won't be hit
                //      - we don't need to check if workspace_dir == comptime_crate_dir because for our purposes it's redundant with comptime_crate_dir == crate_dir
                (Some(workspace_dir), Some(crate_dir), comptime_crate_dir)
                    if workspace_dir != crate_dir && comptime_crate_dir == crate_dir =>
                {
                    "../frontend".to_string()
                }
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
        std::env::var("CRA_MANIFEST_PATH").unwrap_or_else(|_| {
            match (
                cargo_locate_project_workspace(),
                cargo_locate_project(),
                env!("CARGO_MANIFEST_DIR"),
            ) {
                (Some(workspace_dir), Some(crate_dir), comptime_crate_dir)
                    if workspace_dir != crate_dir && comptime_crate_dir == crate_dir =>
                {
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
                cargo_locate_project_workspace(),
                cargo_locate_project(),
                env!("CARGO_MANIFEST_DIR"),
            ) {
                (Some(workspace_dir), Some(crate_dir), comptime_crate_dir)
                    if workspace_dir != crate_dir && comptime_crate_dir == crate_dir =>
                {
                    "views/**/*.html".to_string()
                }
                _ => "backend/views/**/*.html".to_string(),
            }
        })
    })
}
