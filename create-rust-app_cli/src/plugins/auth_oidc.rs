use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use crate::utils::logger::add_file_msg;
use crate::utils::{fs, logger};
use crate::{BackendDatabase, BackendFramework};
use anyhow::Result;
use indoc::indoc;
use rust_embed::RustEmbed;

pub struct AuthOIDC {}

#[derive(RustEmbed)]
#[folder = "template-plugin-auth-oidc"]
struct Asset;

impl Plugin for AuthOIDC {
    fn name(&self) -> &'static str {
        "OIDC Auth"
    }

    fn install(&self, install_config: InstallConfig) -> Result<()> {
        if !install_config.plugin_auth {
            logger::exit_code("Cannot install OIDC Auth plugin without Auth plugin", 1);
        }

        // ===============================
        // COPY TEMPLATE FILES
        // ===============================

        for filename in Asset::iter() {
            let file_contents = Asset::get(filename.as_ref()).unwrap();
            let mut file_path = std::path::PathBuf::from(&install_config.project_dir);
            file_path.push(filename.as_ref());
            let mut directory_path = std::path::PathBuf::from(&file_path);
            directory_path.pop();

            add_file_msg(filename.as_ref());
            std::fs::create_dir_all(directory_path)?;
            std::fs::write(file_path, file_contents.data)?;
        }

        // ===============================
        // New env vars
        // ===============================
        fs::append(
            ".env.example",
            "GOOGLE_OAUTH2_CLIENT_ID=abc\nGOOGLE_OAUTH2_CLIENT_SECRET=123\n",
        )?;

        // ===============================
        // Backend changes
        // ===============================
        match install_config.backend_framework {
            BackendFramework::ActixWeb => {
                fs::replace(
                    "backend/main.rs",
                    r#"app = app.app_data(Data::new(AppConfig {
            app_url: std::env::var("APP_URL").unwrap(),
        }));"#,
                    r#"app = app.app_data(Data::new(AppConfig {
            app_url: std::env::var("APP_URL").unwrap(),
        }));
        app = app.app_data(Data::new(create_rust_app::auth::AuthConfig {
            oidc_providers: vec![create_rust_app::auth::oidc::OIDCProvider::GOOGLE(
                std::env::var("GOOGLE_OAUTH2_CLIENT_ID").unwrap(),
                std::env::var("GOOGLE_OAUTH2_CLIENT_SECRET").unwrap(),
                format!(
                    "{app_url}/oauth/success",
                    app_url = std::env::var("APP_URL").unwrap()
                ),
                format!(
                    "{app_url}/oauth/error",
                    app_url = std::env::var("APP_URL").unwrap()
                ),
            )],
        }));
"#,
                )?;
            }
            BackendFramework::Poem => {
                fs::replace(
                    "backend/main.rs",
                    r#"app = app.app_data(Data::new(AppConfig {
            app_url: std::env::var("APP_URL").unwrap(),
        }));"#,
                    r#"app = app.app_data(Data::new(AppConfig {
            app_url: std::env::var("APP_URL").unwrap(),
        }));
"#,
                )?;

                fs::replace(
                    "backend/main.rs",
                    r#".with(AddData::new(AppConfig {
                    app_url: std::env::var("APP_URL").unwrap(),
                 })"#,
                    r##".with(AddData::new(AppConfig {
                    app_url: std::env::var("APP_URL").unwrap(),
                 })
                 .with(AddData::new(create_rust_app::auth::AuthConfig {
            oidc_providers: vec![create_rust_app::auth::oidc::OIDCProvider::GOOGLE(
                std::env::var("GOOGLE_OAUTH2_CLIENT_ID").unwrap(),
                std::env::var("GOOGLE_OAUTH2_CLIENT_SECRET").unwrap(),
                format!(
                    "{app_url}/oauth/success",
                    app_url = std::env::var("APP_URL").unwrap()
                ),
                format!(
                    "{app_url}/oauth/error",
                    app_url = std::env::var("APP_URL").unwrap()
                ),
            )],
        })"##,
                )?;
            }
        }

        // ===============================
        // MIGRATIONS
        // ===============================

        crate::content::migration::create(
            "plugin_auth-oidc",
            match install_config.backend_database {
                BackendDatabase::Postgres => indoc! {r#"
      CREATE TABLE user_oauth2_links (
        id SERIAL PRIMARY KEY,
        provider TEXT NOT NULL,

        -- all attempts at oauth2 will create a record with these properties
        csrf_token TEXT NOT NULL,
        nonce TEXT NOT NULL,
        pkce_secret TEXT NOT NULL,

        -- when oauth2 attempts succeed, either a user is created or the oauth2 attempt is discarded
        -- depending on whether or not the user ends up linking the account or not
        refresh_token TEXT,
        access_token TEXT,
        subject_id TEXT UNIQUE,
        user_id INT REFERENCES users(id),

        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
      );

      SELECT manage_updated_at('user_oauth2_links');
    "#},
                BackendDatabase::Sqlite => indoc! {r#"
      CREATE TABLE user_oauth2_links (
        id SERIAL PRIMARY KEY,
        provider TEXT NOT NULL,

        -- all attempts at oauth2 will create a record with these properties
        csrf_token TEXT NOT NULL,
        nonce TEXT NOT NULL,
        pkce_secret TEXT NOT NULL,

        -- when oauth2 attempts succeed, either a user is created or the oauth2 attempt is discarded
        -- depending on whether or not the user ends up linking the account or not
        refresh_token TEXT,
        access_token TEXT,
        subject_id TEXT UNIQUE,
        user_id INT REFERENCES users(id),

        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
      );

    "#},
            },
            indoc! {r#"
      DROP TABLE user_oauth2_links;
    "#},
        )?;

        // ===============================
        // PATCH FRONTEND
        // ===============================

        fs::replace(
            "frontend/src/hooks/useAuth.tsx",
            "logout,",
            indoc! {r#"logout,
loginOIDC,
completeOIDCLogin"#},
        )?;

        fs::replace(
            "frontend/src/hooks/useAuth.tsx",
            "const logout = async ()",
            indoc! {r#"
const loginOIDC = async (provider: string) => {
    window.location.href = `/api/auth/oidc/${provider}`
}

const completeOIDCLogin = (): boolean => {
    const params = new URLSearchParams(window.location.search);
    let access_token = params.get('access_token');
    if (!access_token) {
        context.setAccessToken(undefined)
        context.setSession(undefined)

        return false
    } else {
        const parsedToken = parseJwt(access_token) as AccessTokenClaims
        const permissions = new Permissions(parsedToken.roles, parsedToken.permissions)
        context.setAccessToken(access_token)
        context.setSession({
            userId: parsedToken.sub,
            expiresOnUTC: parsedToken.exp,
            roles: permissions.roles,
            permissions: permissions.permissions,
            hasPermission: permissions.hasPermission,
            hasRole: permissions.hasRole,
        })

        return true
    }
}

const logout = async ()"#},
        )?;

        fs::replace(
            "frontend/src/App.tsx",
            "import { LoginPage } from './containers/LoginPage'",
            r#"import { LoginPage } from './containers/LoginPage'
import { OauthLoginResultPage } from './containers/OauthLoginResultPage'"#,
        )?;

        fs::replace(
            "frontend/src/App.tsx",
            r#"<Route path="/login" element={<LoginPage />} />"#,
            r#"<Route path="/login" element={<LoginPage />} />
            <Route path="/oauth/success" element={<OauthLoginResultPage />} />
            <Route path="/oauth/error" element={<OauthLoginResultPage />} />"#,
        )?;

        fs::replace(
            "frontend/src/containers/LoginPage.tsx",
            r#"<div style={{ display: 'flex', flexFlow: 'column' }}>
        <button disabled={processing} onClick={login}>
          Login
        </button>
      </div>"#,
            r##"<div style={{ display: 'flex', flexFlow: 'column' }}>
        <button disabled={processing} onClick={login}>
          Login
        </button>
      </div>
      <a
        style={{ marginTop: '30px' }}
        onClick={() => auth.loginOIDC('google')}
        href="#"
      >
        Login with Google
      </a>
"##,
        )?;

        Ok(())
    }
}
