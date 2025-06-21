use anyhow::Result;
use indoc::indoc;
use rust_embed::RustEmbed;

use crate::content::cargo_toml::add_dependency;
use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use crate::utils::fs;
use crate::utils::logger::add_file_msg;
use crate::{BackendDatabase, BackendFramework};

pub struct Tasks {}

#[derive(RustEmbed)]
#[folder = "template-plugin-tasks"]
struct Asset;

impl Plugin for Tasks {
    fn name(&self) -> &'static str {
        "Tasks"
    }

    fn install(&self, install_config: InstallConfig) -> Result<()> {
        // ===============================
        // Check if the plugin is supported
        // ===============================

        if install_config.backend_database != BackendDatabase::Postgres {
            return Err(anyhow::anyhow!("The tasks plugin only supports Postgres"));
        }

        if install_config.backend_framework != BackendFramework::ActixWeb {
            return Err(anyhow::anyhow!("The tasks plugin only supports actix-web"));
        }

        // ===============================
        // Copy over template files
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
        // Create migration
        // ===============================

        crate::content::migration::create(
            "plugin_tasks",
            indoc! {r#"CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE fang_task_state AS ENUM ('new', 'in_progress', 'failed', 'finished', 'retried');

CREATE TABLE fang_tasks (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    metadata jsonb NOT NULL,
    error_message TEXT,
    state fang_task_state DEFAULT 'new' NOT NULL,
    task_type VARCHAR DEFAULT 'common' NOT NULL,
    uniq_hash CHAR(64),
    retries INTEGER DEFAULT 0 NOT NULL,
    scheduled_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX fang_tasks_state_index ON fang_tasks(state);
CREATE INDEX fang_tasks_type_index ON fang_tasks(task_type);
CREATE INDEX fang_tasks_scheduled_at_index ON fang_tasks(scheduled_at);
CREATE INDEX fang_tasks_uniq_hash ON fang_tasks(uniq_hash);
"#},
            r"DROP TABLE fang_tasks;",
        )?;

        // ===============================
        // Create/update backend files
        // ===============================
        fs::replace("backend/main.rs", "mod mail;", "mod mail;\nmod tasks;")?;

        let example_tasks = r#"

    // Tasks plugin example: sync queue example
    // See fang docs for more info: https://docs.rs/fang/0.10.4/fang/
    {
        // The blocking queue re-uses the app's db connection pool
        let queue = create_rust_app::tasks::queue();

        // An example of how to schedule a blocking task (see `fang` docs for more info):
        use fang::Queueable;
        queue.schedule_task(&tasks::daily_todo::DailyTodo { text: "Call mom (DailyTodo task)".to_string() }).unwrap();
    }

    // Tasks plugin example: async queue example
    // See fang docs for more info: https://docs.rs/fang/0.10.4/fang/
    {
        // The async queue uses a separate db connection pool. We need to connnect it at least once before we can use it throughout out app.
        let mut async_queue = create_rust_app::tasks::async_queue();
        async_queue.lock().unwrap().connect(NoTls).await.expect("Failed to connect to async queue database");
        // this means you need to have the above line somewhere in `main.rs`, before any async jobs are scheduled

        // and here's how you can schedule an async task:
        async_queue.lock().unwrap().schedule_task(&tasks::daily_todo_async::DailyTodoAsync { text: "Call mom (DailyTodoAsync task)".to_string() } as &dyn AsyncRunnable).await.unwrap();
    }

"#.trim();

        match install_config.backend_framework {
            BackendFramework::ActixWeb => {
                fs::replace(
                    "backend/main.rs",
                    "use actix_web::web::Data;",
                    "use actix_web::web::Data;\nuse fang::{AsyncQueueable, AsyncRunnable, NoTls, Queueable};",
                )?;

                fs::replace(
                    "backend/main.rs",
                    "HttpServer::new(move || {",
                    &format!(
                        "{example_tasks}

    HttpServer::new(move || {{"
                    ),
                )?;
            }
            BackendFramework::Poem => {
                fs::replace(
                    "backend/main.rs",
                    "use poem::endpoint::{StaticFilesEndpoint};",
                    "use poem::endpoint::{StaticFilesEndpoint};\nuse fang::{AsyncQueueable, AsyncRunnable, NoTls, Queueable};",
                )?;

                fs::replace(
                    "backend/main.rs",
                    "let data = create_rust_app::setup();",
                    &format!(
                        "{example_tasks}

    let data = create_rust_app::setup();"
                    ),
                )?;
            }
        }

        // ===============================
        // Add dependencies
        // ===============================

        add_dependency(&install_config.project_dir, "fang", r#"fang = "0.10.4""#)?;

        Ok(())
    }
}
