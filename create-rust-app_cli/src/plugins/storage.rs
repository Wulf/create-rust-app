use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use crate::utils::fs;
use crate::utils::logger::add_file_msg;
use crate::{BackendDatabase, BackendFramework};
use anyhow::Result;
use indoc::indoc;
use rust_embed::RustEmbed;

pub struct Storage {}

#[derive(RustEmbed)]
#[folder = "template-plugin-storage"]
struct Asset;

impl Plugin for Storage {
    fn name(&self) -> &'static str {
        "Storage"
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

        fs::append(
            ".env.example",
            r#"
S3_HOST=http://localhost:9000
S3_REGION=minio
S3_BUCKET=bucket
S3_ACCESS_KEY_ID=access_key
S3_SECRET_ACCESS_KEY=secret_key
"#,
        )?;

        fs::replace(
            "frontend/src/App.tsx",
            r#"{/* CRA: routes */}"#,
            r#"{/* CRA: routes */}
            <Route path="/files" element={<Files />} />"#,
        )?;

        fs::replace(
            "frontend/src/App.tsx",
            r#"<a className="NavButton" onClick={() => navigate('/todos')}>Todos</a>"#,
            r#"<a className="NavButton" onClick={() => navigate('/todos')}>Todos</a>
        <a className="NavButton" onClick={() => navigate('/files')}>Files</a>"#,
        )?;

        fs::replace(
            "frontend/src/App.tsx",
            r##"import { Todos } from './containers/Todo'"##,
            r##"import { Todos } from './containers/Todo'
import { Files } from './containers/Files'"##,
        )?;

        crate::content::migration::create(
            "plugin_storage",
            match install_config.backend_database {
                BackendDatabase::Postgres => indoc! {r#"
CREATE TABLE attachment_blobs(
  id SERIAL PRIMARY KEY,

  key TEXT NOT NULL,
  file_name TEXT NOT NULL,
  content_type TEXT,
  byte_size BIGINT NOT NULL,
  checksum TEXT NOT NULL,
  service_name TEXT NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE attachments(
  id SERIAL PRIMARY KEY,

  name TEXT NOT NULL,
  record_type TEXT NOT NULL,
  record_id SERIAL NOT NULL,
  blob_id SERIAL REFERENCES attachment_blobs(id) NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#},
                BackendDatabase::Sqlite => indoc! {r#"
CREATE TABLE attachment_blobs(
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

  key TEXT NOT NULL,
  file_name TEXT NOT NULL,
  content_type TEXT,
  byte_size BIGINT NOT NULL,
  checksum TEXT NOT NULL,
  service_name TEXT NOT NULL,

  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE attachments(
  id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

  name TEXT NOT NULL,
  record_type TEXT NOT NULL,
  record_id INTEGER NOT NULL,
  blob_id INTEGER REFERENCES attachment_blobs(id) NOT NULL,

  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#},
            },
            indoc! {r#"
DROP TABLE attachment_blobs;
DROP TABLE attachments;
"#},
        )?;

        match install_config.backend_framework {
            BackendFramework::ActixWeb => {
                crate::content::service::register_actix(
                    "file",
                    r#"services::file::endpoints(web::scope("/files"))"#,
                )?;

                fs::replace(
                    "backend/main.rs",
                    "app = app.app_data(Data::new(app_data.mailer.clone()));",
                    r#"app = app.app_data(Data::new(app_data.mailer.clone()));
        app = app.app_data(Data::new(app_data.storage.clone()));"#,
                )?;
            }
            BackendFramework::Poem => {
                crate::content::service::register_poem("file", "services::file::api()", "/files")?;

                fs::replace(
                    "backend/main.rs",
                    ".with(AddData::new(data.database))",
                    ".with(AddData::new(data.database))
                .with(AddData::new(data.storage))",
                )?;
            }
        };

        fs::append("backend/services/mod.rs", "pub mod file;")?;

        Ok(())
    }
}
