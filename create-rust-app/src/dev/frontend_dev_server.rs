use std::iter::FromIterator;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

use tokio::sync::broadcast::{Receiver, Sender};

use super::NPM;

use super::{check_exit, DevServerEvent, DevState};

pub async fn start(
    project_dir: &'static str,
    dev_port: u16,
    mut signal_rx: Receiver<DevServerEvent>,
    _dev_server_events_s: Sender<DevServerEvent>,
    state: Arc<Mutex<DevState>>,
) {
    println!("Starting frontend server @ http://localhost:21012/");
    let mut m = state.lock().await;
    m.frontend_server_running = true;
    drop(m);
    let mut child_process = Command::new(NPM)
        .args(["run", "start:dev"])
        .env("DEV_SERVER_PORT", dev_port.to_string())
        .current_dir(PathBuf::from_iter([project_dir, "frontend"]))
        .spawn()
        .unwrap();

    // tokio::fs::create_dir_all(PathBuf::from(".cargo/tmp")).await.unwrap();
    // tokio::fs::write(
    //     PathBuf::from(".cargo/tmp/frontend_pid.txt"),
    //     child_process.id().unwrap_or_default().to_string(),
    // ).await.unwrap();

    while let Ok(event) = signal_rx.recv().await {
        if let DevServerEvent::SHUTDOWN = event {
            println!("Shutting down frontend server...");
            if child_process.id().is_some() {
                child_process.kill().await.unwrap();
            }
            let mut m = state.lock().await;
            m.frontend_server_running = false;
            check_exit(&m);
            drop(m);
            break;
        }
    }
    println!("frontend server stopped.");
}
