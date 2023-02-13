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
    let port = std::env::var("DEV_SERVER_PORT");
    if port.is_err() {
        return;
    }
    let port = port.unwrap();

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
    let port = std::env::var("DEV_SERVER_PORT");
    if port.is_err() {
        return;
    }
    let port = port.unwrap();

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
    let port = std::env::var("DEV_SERVER_PORT");
    if port.is_err() {
        return;
    }
    let port = port.unwrap();

    let url = format!("http://localhost:{port}/backend-up");

    // send event to dev server that the backend is up!
    match reqwest::get(url).await {
        Ok(_) => {}
        Err(_) => {
            println!("WARNING: Could not inform dev server of presence.");
        }
    };
}

#[allow(non_camel_case_types)]
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
    pub fn json(self) -> String {
        match self {
            DevServerEvent::CHECK_MIGRATIONS => json!({
                "type": "_",
            })
            .to_string(),
            DevServerEvent::PendingMigrations(migrations_pending, migrations) => json!({
                "type": "migrationsPending",
                "status": migrations_pending,
                "migrations": migrations
            })
            .to_string(),
            DevServerEvent::MigrationResponse(success, error_message) => json!({
                "type": "migrateResponse",
                "status": success,
                "error": error_message,
            })
            .to_string(),
            DevServerEvent::FeaturesList(list) => json!({
                "type": "featuresList",
                "features": list
            })
            .to_string(),
            DevServerEvent::ViteJSStatus(b) => json!({
                "type": "viteStatus",
                "status": b
            })
            .to_string(),
            DevServerEvent::CompileSuccess(b) => json!({
                "type": "compileStatus",
                "compiled": b
            })
            .to_string(),
            DevServerEvent::BackendCompiling(b) => json!({
                "type": "backendCompiling",
                "compiling": b
            })
            .to_string(),
            DevServerEvent::BackendStatus(b) => json!({
                "type": "backendStatus",
                "status": b
            })
            .to_string(),
            DevServerEvent::BackendRestarting(b) => json!({
                "type": "backendRestarting",
                "status": b
            })
            .to_string(),
            DevServerEvent::SHUTDOWN => json!({
                "type": "backendStatus",
                "status": "false"
            })
            .to_string(),
            DevServerEvent::CompileMessages(msgs) => {
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
    let deps = cargo_toml.dependencies;
    let dep = deps.get("create-rust-app").unwrap_or_else(|| {
        panic!(
            "Expected \"{}\" to list 'create-rust-app' as a dependency.",
            project_dir
        )
    });
    let dep = dep.clone();

    dep.req_features().to_vec()
}

pub fn run_server(project_dir: &'static str) {
    clearscreen::clear().expect("failed to clear screen");

    let features = get_features(project_dir);

    println!("..................................");
    println!(".. Starting development server ...");
    println!("..................................");
    let rt = tokio::runtime::Runtime::new().unwrap();

    let dev_port = std::env::var("DEV_SERVER_PORT")
        .map(|p| {
            p.parse::<u16>()
                .expect("Could not parse DEV_SERVER_PORT to u16")
        })
        .unwrap_or_else(|_| {
            find_free_port(60012..65535)
                .expect("FATAL: Could not find a free port for the development server.")
        });

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
            .await
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
            .await
        });
        tokio::spawn(async move {
            frontend_dev_server::start(
                project_dir,
                dev_port,
                signal_rx,
                dev_server_events_s.clone(),
                state,
            )
            .await
        });

        listen_for_signals(signal_tx).await
    });
}

fn check_exit(state: &DevState) {
    // println!("Checking exit status {:#?}", state);
    if !state.backend_server_running && !state.frontend_server_running && !state.watchexec_running {
        exit(0);
    }
}

async fn listen_for_signals(signal_tx: tokio::sync::broadcast::Sender<DevServerEvent>) {
    let (ev_s, ev_r) = priority::bounded::<Event, Priority>(1024);
    let (er_s, mut er_r) = tokio::sync::mpsc::channel(64);

    // panic on errors
    tokio::spawn(async move {
        while let Some(error) = er_r.recv().await {
            panic!(
                "Error handling process signal:\n==============================\n{:#?}",
                error
            );
        }
    });

    // broadcast signals
    tokio::spawn(async move {
        while let Ok((event, _)) = ev_r.recv().await {
            if event.tags.contains(&Tag::Signal(MainSignal::Terminate))
                || event.tags.contains(&Tag::Signal(MainSignal::Interrupt))
            {
                signal_tx.send(DevServerEvent::SHUTDOWN).unwrap();
            }
        }
    });

    worker(er_s, ev_s).await.unwrap();
}
