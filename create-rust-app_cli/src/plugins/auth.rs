use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use crate::utils::fs;
use crate::utils::logger::add_file_msg;
use crate::{BackendDatabase, BackendFramework};
use anyhow::Result;
use indoc::indoc;
use rust_embed::RustEmbed;

pub struct Auth {}

#[derive(RustEmbed)]
#[folder = "template-plugin-auth"]
struct Asset;

impl Plugin for Auth {
    fn name(&self) -> &'static str {
        "Auth"
    }

    fn install(&self, install_config: InstallConfig) -> Result<()> {
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
        // PATCH FRONTEND
        // ===============================

        // TODO: Fix these appends/prepends by prepending the filepath with project_dir
        // currently, this works because we assume the current working directory is the project's root
        fs::prepend(
            "frontend/src/App.tsx",
            r#"import { useAuth, useAuthCheck } from './hooks/useAuth'
import { AccountPage } from './containers/AccountPage'
import { LoginPage } from './containers/LoginPage'
import { ActivationPage } from './containers/ActivationPage'
import { RegistrationPage } from './containers/RegistrationPage'
import { RecoveryPage } from './containers/RecoveryPage'
import { ResetPage } from './containers/ResetPage'"#,
        )?;
        fs::prepend(
            "frontend/bundles/index.tsx",
            "import { AuthProvider } from '../src/hooks/useAuth'",
        )?;
        fs::replace(
            "frontend/src/App.tsx",
            "const App = () => {",
            r#"const App = () => {
  useAuthCheck()
  const auth = useAuth()
    "#,
        )?;
        fs::replace(
            "frontend/src/App.tsx",
            r#"{/* CRA: routes */}"#,
            r#"{/* CRA: routes */}
            <Route path="/login" element={<LoginPage />} />
            <Route path="/recovery" element={<RecoveryPage />} />
            <Route path="/reset" element={<ResetPage />} />
            <Route path="/activate" element={<ActivationPage />} />
            <Route path="/register" element={<RegistrationPage />} />
            <Route path="/account" element={<AccountPage />} />
    "#,
        )?;
        fs::replace(
            "frontend/src/App.tsx",
            "{/* CRA: left-aligned nav buttons */}",
            r#"{/* CRA: left-aligned nav buttons */}
          <a className="NavButton" onClick={() => navigate('/account')}>Account</a>"#,
        )?;
        fs::replace(
            "frontend/src/App.tsx",
            "{/* CRA: right-aligned nav buttons */}",
            r#"{/* CRA: right-aligned nav buttons */}
          { auth.isAuthenticated && <a className="NavButton" onClick={() => auth.logout()}>Logout</a> }
          { !auth.isAuthenticated && <a className="NavButton" onClick={() => navigate('/login')}>Login/Register</a> }"#,
        )?;
        fs::replace(
            "frontend/bundles/index.tsx",
            "{/* CRA: Wrap */}",
            "{/* CRA: Wrap */}\n<AuthProvider>",
        )?;
        fs::replace(
            "frontend/bundles/index.tsx",
            "{/* CRA: Unwrap */}",
            "{/* CRA: Unwrap */}\n</AuthProvider>",
        )?;

        crate::content::migration::create(
            "plugin_auth",
            match install_config.backend_database {
                BackendDatabase::Postgres => indoc! {r#"
      CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        email TEXT NOT NULL,
        hash_password TEXT NOT NULL,
        activated BOOL NOT NULL DEFAULT FALSE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
      );

      SELECT manage_updated_at('users');

      CREATE TABLE user_sessions (
        id SERIAL PRIMARY KEY,
        user_id SERIAL NOT NULL REFERENCES users(id),
        refresh_token TEXT NOT NULL,
        device TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
      );

      SELECT manage_updated_at('user_sessions');

      CREATE TABLE user_permissions (
        user_id SERIAL NOT NULL REFERENCES users(id),
        permission TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (user_id, permission)
      );

      CREATE TABLE user_roles (
        user_id SERIAL NOT NULL REFERENCES users(id),
        role TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (user_id, role)
      );

      CREATE TABLE role_permissions (
        role TEXT NOT NULL,
        permission TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (role, permission)
      );
    "#},
                BackendDatabase::Sqlite => indoc! {r#"
      CREATE TABLE users (
        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        email TEXT NOT NULL,
        hash_password TEXT NOT NULL,
        activated BOOLEAN NOT NULL DEFAULT FALSE,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
      );

      CREATE TABLE user_sessions (
        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        user_id INTEGER NOT NULL REFERENCES users(id),
        refresh_token TEXT NOT NULL,
        device TEXT,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
      );

      CREATE TABLE user_permissions (
        user_id INTEGER NOT NULL REFERENCES users(id),
        permission TEXT NOT NULL,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (user_id, permission)
      );

      CREATE TABLE user_roles (
        user_id INTEGER NOT NULL REFERENCES users(id),
        role TEXT NOT NULL,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (user_id, role)
      );

      CREATE TABLE role_permissions (
        role TEXT NOT NULL,
        permission TEXT NOT NULL,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (role, permission)
      );
    "#},
            },
            indoc! {r#"
      DROP TABLE user_permissions;
      DROP TABLE role_permissions;
      DROP TABLE user_roles;
      DROP TABLE user_sessions;
      DROP TABLE users;
    "#},
        )?;

        match install_config.backend_framework {
            BackendFramework::ActixWeb => crate::content::service::register_actix(
                "auth",
                r#"create_rust_app::auth::endpoints(web::scope("/auth"))"#,
            )?,
            BackendFramework::Poem => crate::content::service::register_poem(
                "auth",
                "create_rust_app::auth::api()",
                "/auth",
            )?,
        };

        Ok(())
    }
}
