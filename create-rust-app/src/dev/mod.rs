mod backend_compiling_server;
mod dev_server;
mod frontend_dev_server;

pub mod controller;
use cargo_metadata::CompilerMessage;
use cargo_toml::Manifest;
use serde::Serialize;
use serde_json::json;
mod endpoints;
use crate::util::net::find_free_port;
use async_priority_channel as priority;
pub use endpoints::*;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;
use watchexec::event::{Event, Priority, Tag};
use watchexec::signal::source::worker;
use watchexec::signal::source::MainSignal;

#[cfg(windows)]
const NPM: &str = "npm.cmd";

#[cfg(not(windows))]
const NPM: &str = "npm";

pub async fn vitejs_ping_down() {
    let Ok(port) = std::env::var("DEV_SERVER_PORT") else {
        return;
    };

    let url = format!("http://localhost:{port}/vitejs-down");

    // send event to dev server that the backend is up!
    match reqwest::get(url).await {
        Ok(_) => {}
        Err(_) => {
            println!("WARNING: Could not inform dev server that vitejs is down.");
        }
    };
}

pub async fn vitejs_ping_up() {
    let Ok(port) = std::env::var("DEV_SERVER_PORT") else {
        return;
    };

    let url = format!("http://localhost:{port}/vitejs-up");

    // send event to dev server that the backend is up!
    match reqwest::get(url).await {
        Ok(_) => {}
        Err(_) => {
            println!("WARNING: Could not inform dev server that vitejs is up.");
        }
    };
}

pub async fn setup_development() {
    let Ok(port) = std::env::var("DEV_SERVER_PORT") else {
        return;
    };

    let url = format!("http://localhost:{port}/backend-up");

    // send event to dev server that the backend is up!
    match reqwest::get(url).await {
        Ok(_) => {}
        Err(_) => {
            println!("WARNING: Could not inform dev server of presence.");
        }
    };
}

#[allow(non_camel_case_types, clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub enum DevServerEvent {
    /*
        These events are internal: they shouldn't be used to communicate with the web browser client
    */
    /// sent when Ctrl+C or similar signals are sent to the dev server parent process
    SHUTDOWN, // related to external event: "BackendStatus"
    /// sent once when the backend compiles successfully and more when files in the migrations/ directory change
    CHECK_MIGRATIONS, // related to external event: "PendingMigrations"

    /*
        These events are external: they are sent to the web browser client
    */
    PendingMigrations(bool, Vec<CreateRustAppMigration>),
    MigrationResponse(bool, Option<String>),
    FeaturesList(Vec<String>),
    ViteJSStatus(bool),
    BackendCompiling(bool),
    BackendRestarting(bool),
    BackendStatus(bool),
    CompileSuccess(bool),
    CompileMessages(Vec<CompilerMessage>),
}

#[derive(Serialize, Debug, Clone)]
pub enum MigrationStatus {
    Applied,
    Pending,
    AppliedButMissingLocally,
    Unknown,
}

#[derive(Serialize, Debug, Clone)]
pub struct CreateRustAppMigration {
    name: String,
    version: String,
    status: MigrationStatus,
}

impl DevServerEvent {
    /// # Panics
    /// * if the event cannot be serialized to JSON
    #[must_use]
    pub fn json(self) -> String {
        match self {
            Self::CHECK_MIGRATIONS => json!({
                "type": "_",
            })
            .to_string(),
            Self::PendingMigrations(migrations_pending, migrations) => json!({
                "type": "migrationsPending",
                "status": migrations_pending,
                "migrations": migrations
            })
            .to_string(),
            Self::MigrationResponse(success, error_message) => json!({
                "type": "migrateResponse",
                "status": success,
                "error": error_message,
            })
            .to_string(),
            Self::FeaturesList(list) => json!({
                "type": "featuresList",
                "features": list
            })
            .to_string(),
            Self::ViteJSStatus(b) => json!({
                "type": "viteStatus",
                "status": b
            })
            .to_string(),
            Self::CompileSuccess(b) => json!({
                "type": "compileStatus",
                "compiled": b
            })
            .to_string(),
            Self::BackendCompiling(b) => json!({
                "type": "backendCompiling",
                "compiling": b
            })
            .to_string(),
            Self::BackendStatus(b) => json!({
                "type": "backendStatus",
                "status": b
            })
            .to_string(),
            Self::BackendRestarting(b) => json!({
                "type": "backendRestarting",
                "status": b
            })
            .to_string(),
            Self::SHUTDOWN => json!({
                "type": "backendStatus",
                "status": "false"
            })
            .to_string(),
            Self::CompileMessages(msgs) => {
                let messages = serde_json::to_value(&msgs).unwrap();
                json!({
                    "type": "compilerMessages",
                    "messages": messages
                })
                .to_string()
            }
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct DevState {
    pub frontend_server_running: bool,
    pub backend_server_running: bool,
    pub watchexec_running: bool,
}

fn get_features(project_dir: &'static str) -> Vec<String> {
    let cargo_toml = Manifest::from_path(PathBuf::from_iter([project_dir, "Cargo.toml"]))
        .unwrap_or_else(|_| panic!("Could not find \"{}\"", project_dir));
    // .expect(&format!("Could not find \"{project_dir}\""));
    let mut deps = cargo_toml.dependencies;
    if let Some(workspace) = cargo_toml.workspace {
        // if the manifest has a workspace table, also read dependencies in there
        deps.extend(workspace.dependencies);
    }
    let dep = deps.get("create-rust-app").unwrap_or_else(|| {
        panic!(
            "Expected \"{}\" to list 'create-rust-app' as a dependency.",
            project_dir
        )
    });
    let dep = dep.clone();

    dep.req_features().to_vec()
}

/// # Panics
/// * if the project directory doesn't exist
/// * if the project directory doesn't contain a Cargo.toml file
/// * if the Cargo.toml file doesn't list `create-rust-app` as a dependency
/// * cannot start a tokio runtime
pub fn run_server(project_dir: &'static str) {
    clearscreen::clear().expect("failed to clear screen");

    let features = get_features(project_dir);

    println!("..................................");
    println!(".. Starting development server ...");
    println!("..................................");
    let rt = tokio::runtime::Runtime::new().unwrap();

    let dev_port = std::env::var("DEV_SERVER_PORT").map_or_else(
        |_| {
            find_free_port(60012..65535)
                .expect("FATAL: Could not find a free port for the development server.")
        },
        |p| {
            p.parse::<u16>()
                .expect("Could not parse DEV_SERVER_PORT to u16")
        },
    );

    rt.block_on(async move {
        let state = Arc::new(Mutex::new(DevState {
            backend_server_running: false,
            frontend_server_running: false,
            watchexec_running: false,
        }));
        let state2 = state.clone();

        // used for shutdown only (TODO: merge this with next broadcast channel)
        let (signal_tx, signal_rx) = tokio::sync::broadcast::channel::<DevServerEvent>(64);
        let signal_rx2 = signal_tx.subscribe();

        // used for websocket events
        let (dev_server_events_s, dev_server_events_r) =
            tokio::sync::broadcast::channel::<DevServerEvent>(64);
        let dev_server_events_s2 = dev_server_events_s.clone();
        let dev_server_events_s3 = dev_server_events_s.clone();

        // HACK: used to ignore some file modification events as a result of interaction with the dev server
        let (file_events_s, _) = tokio::sync::broadcast::channel::<String>(64);
        let file_events_s2 = file_events_s.clone();

        tokio::spawn(async move {
            dev_server::start(
                project_dir,
                dev_port,
                dev_server_events_r,
                dev_server_events_s3,
                file_events_s2,
                features,
            )
            .await;
        });
        tokio::spawn(async move {
            backend_compiling_server::start(
                project_dir,
                dev_port,
                signal_rx2,
                dev_server_events_s2,
                state2,
                file_events_s,
            )
            .await;
        });
        tokio::spawn(async move {
            frontend_dev_server::start(
                project_dir,
                dev_port,
                signal_rx,
                dev_server_events_s.clone(),
                state,
            )
            .await;
        });

        listen_for_signals(signal_tx).await;
    });
}

fn check_exit(state: &DevState) {
    // println!("Checking exit status {:#?}", state);
    if !state.backend_server_running && !state.frontend_server_running && !state.watchexec_running {
        exit(0);
    }
}

async fn listen_for_signals(signal_tx: tokio::sync::broadcast::Sender<DevServerEvent>) {
    let (event_sender, event_receiver) = priority::bounded::<Event, Priority>(1024);
    let (error_sender, mut error_receiver) = tokio::sync::mpsc::channel(64);

    // panic on errors
    tokio::spawn(async move {
        // we panic on the first error we receive, so we only need to recv once
        // if, in the future, we change the error handling to be more robust (not panic), we'll need to loop here
        // like `while let Some(error) = error_receiver.recv().await { ... }`
        if let Some(error) = error_receiver.recv().await {
            panic!(
                "Error handling process signal:\n==============================\n{:#?}",
                error
            );
        }
    });

    // broadcast signals
    tokio::spawn(async move {
        while let Ok((event, _)) = event_receiver.recv().await {
            if event.tags.contains(&Tag::Signal(MainSignal::Terminate))
                || event.tags.contains(&Tag::Signal(MainSignal::Interrupt))
            {
                signal_tx.send(DevServerEvent::SHUTDOWN).unwrap();
            }
        }
    });

    worker(error_sender, event_sender).await.unwrap();
}
