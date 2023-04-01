use cargo_metadata::Message;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc;
use watchexec::action::Action;
use watchexec::config::{InitConfig, RuntimeConfig};
use watchexec::event::{Event, Priority, Tag};
use watchexec::handler::PrintDebug;
use watchexec::signal::source::MainSignal;
use watchexec::Watchexec;

use super::{check_exit, DevServerEvent, DevState};

pub async fn start(
    project_dir: &'static str,
    dev_port: u16,
    mut signal_rx: Receiver<DevServerEvent>, // this is specifically for the SHUTDOWN signal
    dev_server_events_s: Sender<DevServerEvent>, // this one is for any dev server event signal
    state: Arc<Mutex<DevState>>,
    file_events_s: Sender<String>,
) {
    println!("Starting backend server @ http://localhost:3000/");
    let state2 = state.clone();
    let state3 = state.clone();

    let (server_s, mut server_r) = mpsc::channel::<&str>(64);
    let server_s2 = server_s.clone();

    let ws2_s = dev_server_events_s.clone();
    // let ws3_s = dev_server_events_s.clone();

    tokio::spawn(async move {
        let mut m = state.lock().await;
        m.backend_server_running = true;
        drop(m);

        let new_child_process = || {
            Command::new("cargo")
                .arg("run")
                .kill_on_drop(true)
                .env("DEV_SERVER_PORT", dev_port.to_string())
                .current_dir(project_dir)
                .spawn()
                .unwrap()
        };

        let mut child_process = new_child_process();
        ws2_s.send(DevServerEvent::CHECK_MIGRATIONS).ok();

        while let Some(event) = server_r.recv().await {
            match event {
                "restart" => {
                    println!("♻️  Restarting server...");
                    // we don't send BackendStatus(false) but instead we notify the user that it's restarting which should be enough
                    // ws2_s.send(DevServerEvent::BackendStatus(false)).ok();
                    ws2_s.send(DevServerEvent::BackendRestarting(true)).ok();
                    if child_process.id().is_some() {
                        child_process.kill().await.unwrap();
                    }
                    child_process = new_child_process();
                    // note: the child process will hit /backend-up which
                    //       sends backend status "true" to the websocket
                    //       i.e. ws2_s.send(DevServerEvent::BackendStatus(true)).ok();
                    //            is not necessary
                }
                "stop" => {
                    println!("Shutting down backend server...");
                    ws2_s.send(DevServerEvent::BackendStatus(false)).ok();
                    if child_process.id().is_some() {
                        child_process.kill().await.unwrap();
                    }
                    let mut m = state.lock().await;
                    m.backend_server_running = false;
                    check_exit(&m);
                    drop(m);
                    break;
                }
                _ => {}
            }
        }
    });

    let mut init = InitConfig::default();
    init.on_error(PrintDebug(std::io::stderr()));

    let mut runtime = RuntimeConfig::default();
    // runtime.command(watchexec::command::Command::Exec {
    //     prog: "cargo".to_string(),
    //     args: vec!["run".to_string()],
    // });
    let mut file_events_r = file_events_s.subscribe();
    let files_to_ignore = Arc::new(std::sync::Mutex::new(vec![]));
    let files_to_ignore2 = files_to_ignore.clone();
    tokio::spawn(async move {
        while let Ok(file) = file_events_r.recv().await {
            let mut arr = files_to_ignore2.lock().unwrap();
            arr.push(file);
        }
    });
    let backend_dir = PathBuf::from(format!("{project_dir}/backend"));
    let migrations_dir = PathBuf::from(format!("{project_dir}/migrations"));
    runtime.pathset([backend_dir, migrations_dir.clone()]);
    runtime.on_action(move |action: Action| {
        let files_to_ignore = files_to_ignore.clone();
        let server_s2 = server_s2.clone();
        let state3 = state3.clone();
        let ws_s = dev_server_events_s.clone();
        let migrations_dir = migrations_dir.clone();

        async move {
            // no exit events
            if action
                .events
                .iter()
                .any(|e| e.metadata.contains_key("exit-watchexec"))
            {
                println!("continuous backend compilation stopped.");
                let mut m = state3.lock().await;
                m.watchexec_running = false;
                check_exit(&m);
                drop(m);
                return Ok(());
            }

            let mut ignored_files = files_to_ignore.lock().unwrap();
            let files_to_ignore: Vec<String> = ignored_files.clone();
            ignored_files.clear();
            drop(ignored_files);

            let mut touched_migrations_dir = false;

            // no file events
            if action.events.iter().any(|e| {
                e.tags.iter().any(|t| match t {
                    Tag::Path { path, file_type: _ } => {
                        if path
                            .to_str()
                            .unwrap()
                            .starts_with(migrations_dir.as_os_str().to_str().unwrap())
                        {
                            touched_migrations_dir = true;
                        }

                        !files_to_ignore
                            .iter()
                            .any(|file_to_ignore| path.to_str().unwrap().ends_with(file_to_ignore))
                    }
                    _ => false,
                })
            }) {
                // compile
                ws_s.send(DevServerEvent::BackendCompiling(true)).ok();
                if compile(project_dir, ws_s.clone()) {
                    // restart backend
                    server_s2.send("restart").await.unwrap();
                    ws_s.send(DevServerEvent::CompileSuccess(true)).ok();
                } else {
                    ws_s.send(DevServerEvent::CompileSuccess(false)).ok();
                }
                ws_s.send(DevServerEvent::BackendCompiling(false)).ok();
            }

            if touched_migrations_dir {
                ws_s.send(DevServerEvent::CHECK_MIGRATIONS).ok();
            }

            Ok::<(), std::io::Error>(())
        }
    });

    let we = Watchexec::new(init, runtime.clone()).unwrap();
    let we2 = we.clone();

    tokio::spawn(async move {
        while let Ok(event) = signal_rx.recv().await {
            if let DevServerEvent::SHUTDOWN = event {
                let mut metadata = HashMap::new();
                metadata.insert("exit-watchexec".to_string(), vec!["true".to_string()]);
                we2.send_event(
                    Event {
                        tags: vec![
                            Tag::Signal(MainSignal::Interrupt),
                            Tag::Signal(MainSignal::Terminate),
                        ],
                        metadata,
                    },
                    Priority::Urgent,
                )
                .await
                .unwrap(); // stops watch exec
                server_s.send("stop").await.unwrap(); // stops backend server
            }
        }
    });

    let mut m = state2.lock().await;
    m.watchexec_running = true;
    drop(m);

    we.main().await.unwrap().unwrap();
    println!("backend compilation server stopped.");
}

fn compile(project_dir: &'static str, ws_s: Sender<DevServerEvent>) -> bool {
    println!("🔨 Compiling backend...");
    let start_time = std::time::SystemTime::now();

    let mut command = std::process::Command::new("cargo")
        .args([
            "build",
            "-q",
            "--message-format=json-diagnostic-rendered-ansi",
        ])
        .current_dir(project_dir)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let reader = std::io::BufReader::new(command.stdout.take().unwrap());
    let mut compiler_messages = vec![];
    ws_s.send(DevServerEvent::CompileMessages(compiler_messages.clone()))
        .ok(); // clear previous messages
    for message in cargo_metadata::Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerMessage(msg) => {
                compiler_messages.push(msg);
                ws_s.send(DevServerEvent::CompileMessages(compiler_messages.clone()))
                    .ok();
            }
            Message::CompilerArtifact(_) => {
                // println!("{:?}", artifact);
            }
            Message::BuildScriptExecuted(_) => {
                // println!("{:?}", script);
            }
            Message::BuildFinished(finished) => {
                let compile_time_s = std::time::SystemTime::now()
                    .duration_since(start_time)
                    .map(|d| d.as_secs_f32())
                    .map(|d| format!("{d:.2}"))
                    .unwrap_or_else(|_| "?".to_string());

                if finished.success {
                    println!("✅ Compiled ({compile_time_s} seconds)");
                } else {
                    println!("❌ Compilation failed: see errors in app ({compile_time_s} seconds)",);
                }
            }
            _ => (), // Unknown message
        }
    }

    command
        .wait_with_output()
        .expect("Error retrieving `cargo build` exit status")
        .status
        .success()
}
