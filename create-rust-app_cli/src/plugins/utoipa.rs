use crate::content::cargo_toml::add_dependency;
use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use crate::utils::logger::register_service_msg;
use crate::{fs, BackendFramework};
use anyhow::Result;

pub struct Utoipa {}

impl Plugin for Utoipa {
    fn name(&self) -> &'static str {
        "Utoipa"
    }

    fn install(&self, install_config: InstallConfig) -> Result<()> {
        // TODO: remove when utoipa plugin is implemented for Poem
        if install_config.backend_framework != BackendFramework::ActixWeb {
            crate::logger::error("Currently, the Utoipa plugin requires the Actix plugin!");
            std::process::exit(1);
        }

        // add link to Swagger UI docs
        fs::replace(
            "frontend/src/App.tsx",
            r"{/* CRA: right-aligned nav buttons */}",
            r#"{/* CRA: right-aligned nav buttons */}
            <a className="NavButton" onClick={() => window.location.href = "/swagger-ui/" }>API</a>"#,
        )?;

        // add dependencies
        match install_config.backend_framework {
            BackendFramework::ActixWeb => {
                add_dependency(
                    &install_config.project_dir,
                    "utoipa",
                    r#"utoipa = { version="4", features=["actix_extras", "chrono", "openapi_extensions"] }"#,
                )?;
                add_dependency(
                    &install_config.project_dir,
                    "utoipa-swagger-ui",
                    r#"utoipa-swagger-ui = { version="4", features=["actix-web"]}"#,
                )?;
            }
            BackendFramework::Poem => {
                crate::logger::error(
                    "plugin_utoipa not yet implemented for backends other than actix-web",
                );
                std::process::exit(1);
            }
        }

        // add service to main.rs
        match install_config.backend_framework {
            BackendFramework::ActixWeb => {
                register_service_msg("(dev-only) /utoipa");

                fs::replace(
                    "backend/main.rs",
                    "/* Development-only routes */",
                    r#"/* Development-only routes */
            
            /* Mount Swagger ui */
            use utoipa::OpenApi;
            use utoipa_swagger_ui::{SwaggerUi, Url};
            app = app.service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
                (
                     Url::new("auth", "/api-doc/openapi_auth.json"),
                     create_rust_app::auth::ApiDoc::openapi(),
                ),
            ]));"#,
                )?;
            }
            BackendFramework::Poem => {
                panic!("plugin_utoipa not yet implemented for backends other than actix-web");
            }
        };

        Ok(())
    }
}
