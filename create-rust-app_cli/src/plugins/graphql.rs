use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use crate::utils::logger::add_file_msg;
use anyhow::Result;
use rust_embed::RustEmbed;
use crate::{BackendFramework, fs};
use crate::content::cargo_toml::add_dependency;
use crate::logger::{register_service_msg};

pub struct GraphQL {}

#[derive(RustEmbed)]
#[folder = "template-plugin-graphql"]
struct Asset;

impl Plugin for GraphQL {
    fn name(&self) -> &'static str {
        "Storage"
    }

    fn install(&self, install_config: InstallConfig) -> Result<()> {
        if !install_config.plugin_auth {
            crate::logger::error("The GraphQL plugin requires the Auth plugin!");
            std::process::exit(1);
        }

        for filename in Asset::iter() {
            let file_contents = Asset::get(filename.as_ref()).unwrap();
            let mut file_path = std::path::PathBuf::from(&install_config.project_dir);
            file_path.push(filename.as_ref());
            let mut directory_path = std::path::PathBuf::from(&file_path);
            directory_path.pop();

            add_file_msg(filename.as_ref());
            std::fs::create_dir_all(directory_path)?;
            std::fs::write(file_path, file_contents)?;
        }

        add_dependency(&install_config.project_dir, "async-graphql", r#"async-graphql = "3.0.38""#)?;
        add_dependency(&install_config.project_dir, "jsonwebtoken", r#"jsonwebtoken = "8.1.0""#)?;
        match install_config.backend_framework {
            BackendFramework::ActixWeb => {
                add_dependency(&install_config.project_dir, "async-graphql-actix-web", r#"async-graphql-actix-web = "3.0.38""#)?;
            },
            BackendFramework::Poem => {
                add_dependency(&install_config.project_dir, "async-graphql-poem", r#"async-graphql-poem = "3.0.38""#)?;
            }
        }

        fs::prepend(
            "frontend/src/App.tsx",
            r#"import { useApolloClient } from '@apollo/client'
import { GraphQLPage } from './containers/GraphQLPage'"#,
        )?;

        fs::replace("frontend/src/App.tsx", "/* CRA: app hooks */", "/* CRA: app hooks */\n  const apollo = useApolloClient()")?;

        fs::replace(
            "frontend/src/App.tsx",
            "{/* CRA: left-aligned nav buttons */}",
            r#"{/* CRA: left-aligned nav buttons */}
          <a className="NavButton" onClick={() => history.push('/gql')}>GraphQL</a>"#,
        )?;

        fs::replace("frontend/src/index.tsx", "ReactDOM.render(", r##"import {ApolloProvider} from "@apollo/client";
import {useAuthenticatedApolloClient} from "./hooks/useAuthenticatedApolloClient";

const AuthenticatedApolloProvider = (props: { children: React.ReactNode }) => {
    const client = useAuthenticatedApolloClient()

    return <ApolloProvider client={client}>
        {props.children}
    </ApolloProvider>
}

ReactDOM.render("##)?;

        fs::replace(
            "frontend/src/App.tsx",
            "{/* CRA: routes */}",
            r#"{/* CRA: routes */}
        <Route path={"/gql"}><GraphQLPage /></Route>"#,
        )?;

        fs::replace("frontend/package.json", r##""dependencies": {"##, r##""dependencies": {
    "@apollo/client": "^3.5.10",
    "graphql-ws": "^5.6.4",
    "graphql": "^16.3.0","##)?;

        fs::replace("backend/main.rs", "mod mail;", "mod mail;\nmod graphql;")?;

        // update auth plugin's logout button
        let old_logout_link = r##"{ auth.isAuthenticated && <a className="NavButton" onClick={() => auth.logout()}>Logout</a> }"##;
        let new_logout_link = r##"{ auth.isAuthenticated && <a className="NavButton" onClick={() => { auth.logout(); apollo.resetStore(); }}>Logout</a> }"##;
        fs::replace("frontend/src/App.tsx", old_logout_link, new_logout_link)?;

        // make sure auth plugin is wrapped on top
        fs::replace("frontend/src/index.tsx", "<AuthProvider>", "<AuthProvider>\n        <AuthenticatedApolloProvider>")?;
        fs::replace("frontend/src/index.tsx", "</AuthProvider>", "</AuthenticatedApolloProvider>\n      </AuthProvider>")?;


        match install_config.backend_framework {
            BackendFramework::ActixWeb => {
                fs::replace("backend/main.rs", "extern crate diesel;", r##"extern crate diesel;

use actix_web::guard;"##)?;

                // GraphQL subscription endpoint
                //
                crate::content::service::register_actix("graphql-websocket",
                    r##"web::resource("/graphql/ws")
                .guard(guard::Get())
                .guard(guard::Header("upgrade", "websocket"))
                .to(graphql::index_ws)"##
                )?;

                // GraphQL query endpoint
                //
                crate::content::service::register_actix(
                    "graphql",
                    r#"web::resource("/graphql").guard(guard::Post()).to(graphql::index)"#
                )?;

                // GraphQL Schema building
                //
                fs::replace("backend/main.rs", "create_rust_app::setup();", r##"create_rust_app::setup();

    let schema = async_graphql::Schema::build(graphql::QueryRoot, graphql::MutationRoot, graphql::SubscriptionRoot)
        .data(app_data.database.clone())
        .data(app_data.mailer.clone())
        .data(app_data.storage.clone())
        .finish();
"##)?;

                // GraphQL Playground endpoint
                //
                register_service_msg("graphql-playground");
                fs::replace("backend/main.rs",
                            "let app = app.app_data(Data::new(app_data.mailer.clone()));",
                            r#"let app = app.app_data(Data::new(app_data.mailer.clone()));
        let app = app.app_data(Data::new(schema.clone()));

        // Enable the GraphQL playground in development
        #[cfg(debug_assertions)]
        let app = app.service(web::resource("/graphql").guard(guard::Get()).to(graphql::index_playground));
"#)?;

            },
            BackendFramework::Poem => {
                // GraphQL Schema building
                //
                fs::replace("backend/main.rs", "create_rust_app::setup();", r##"create_rust_app::setup();

    let schema = async_graphql::Schema::build(graphql::QueryRoot, graphql::MutationRoot, graphql::SubscriptionRoot)
        .data(data.database.clone())
        .data(data.mailer.clone())
        .data(data.storage.clone())
        .finish();
"##)?;

                // GraphQL subscription + query endpoints
                //
                fs::replace("backend/main.rs", "let mut api = Route::new()", r#"let mut api = Route::new()
		.at("/graphql/ws", poem::get(graphql::index_ws))
        .at("/graphql", poem::post(graphql::index))"#)?;

                // GraphQL Playground endpoint
                //
                fs::replace("backend/main.rs", "let mut app = Route::new()", r#"let mut app = Route::new().at("/graphql", poem::get(graphql::playground))"#)?;

                // Adding Schema to exposed data
                //
                fs::replace("backend/main.rs",
                            ".with(AddData::new(data.database))",
                            ".with(AddData::new(data.database))
                .with(AddData::new(schema))"
                )?;
            },
        };


        Ok(())
    }
}
