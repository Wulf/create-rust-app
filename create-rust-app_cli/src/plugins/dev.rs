use crate::logger::register_service_msg;
use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use crate::utils::fs;
use crate::utils::logger::add_file_msg;
use crate::BackendFramework;
use anyhow::Result;
use indoc::indoc;
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
            if filename.starts_with("README.md")
                || filename.contains(".cargo/admin") && !filename.contains(".cargo/admin/dist")
            {
                continue;
            }

            let file_contents = Asset::get(filename.as_ref()).unwrap();
            let mut file_path = std::path::PathBuf::from(&install_config.project_dir);
            file_path.push(filename.as_ref());
            let mut directory_path = std::path::PathBuf::from(&file_path);
            directory_path.pop();

            add_file_msg(filename.as_ref());
            std::fs::create_dir_all(directory_path)?;
            std::fs::write(file_path, file_contents.data)?;
        }

        // TODO: Fix these appends/prepends by prepending the filepath with project_dir
        // currently, this works because we assume the current working directory is the project's root

        // TODO: don't use concurrently as the anchor for new frontend dependencies
        fs::replace(
            "frontend/package.json",
            r#""concurrently": "^7.1.0""#,
            r#""concurrently": "^7.1.0",
    "react-query": "^3.21.0""#,
        )?;

        fs::append(
            "frontend/src/dev.tsx",
            indoc! {r##"
            // Sets up the development environment.
            //
            // Note: When running `cargo frontend` and `cargo backend` individually, "DEV_SERVER_PORT" is not set.
            //       Use `cargo fullstack` for the full development experience.
            if (import.meta.env.DEV_SERVER_PORT) {
                import('./setupDevelopment')
            }
        "##},
        )?;

        match install_config.backend_framework {
            BackendFramework::ActixWeb => {
                register_service_msg("(dev-only) /development");
                register_service_msg("(dev-only) /admin");
                fs::replace(
                    "backend/main.rs",
                    r#"/* Development-only routes */"#,
                    r#"/* Development-only routes */
            // Mount development-only API routes
            api_scope = api_scope.service(create_rust_app::dev::endpoints(web::scope("/development")));
            // Mount the admin dashboard on /admin
            app = app.service(web::scope("/admin").service(Files::new("/", ".cargo/admin/dist/").index_file("admin.html")));"#,
                )?;
            }
            BackendFramework::Poem => {
                register_service_msg("(dev-only) /development");
                register_service_msg("(dev-only) /admin");
                fs::replace(
                    "backend/main.rs",
                    r#"/* Development-only routes */"#,
                    r#"/* Development-only routes */
        // Mount development-only API routes
        api_routes = api_routes.nest("/development", create_rust_app::dev::api());
        // Mount the admin dashboard on /admin
        app = app.at("/admin", StaticFilesEndpoint::new(".cargo/admin/dist").index_file("admin.html"));"#,
                )?;
            }
        }

        Ok(())
    }
}
