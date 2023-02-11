use std::sync::{Arc, Mutex};

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::status::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use cargo_metadata::CompilerMessage;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{dev::controller, Database};

use super::{CreateRustAppMigration, DevServerEvent};

#[derive(Debug)]
struct CurrentDevState {
    backend_status: bool,
    backend_compiled: bool,
    backend_compiling: bool,
    backend_restarting: bool,
    compiler_messages: Vec<CompilerMessage>,
    vite_status: bool,
    features: Vec<String>,
    migrations_pending: (bool, Vec<CreateRustAppMigration>),
}

struct AppState {
    #[allow(dead_code)]
    project_dir: &'static str,
    rx: tokio::sync::Mutex<Receiver<DevServerEvent>>, // this is the original subscribed receiver which hasn't missed a single event :)
    tx: Sender<DevServerEvent>,
    file_tx: Sender<String>,
    db: Database,
    dev: Mutex<CurrentDevState>,
}

pub async fn start(
    project_dir: &'static str,
    dev_port: u16,
    dev_server_events_r: Receiver<DevServerEvent>,
    dev_server_events_s: Sender<DevServerEvent>,
    file_events_s: Sender<String>,
    features: Vec<String>,
) {
    if dotenv::dotenv().is_err() {
        panic!("ERROR: Could not load environment variables from dotenv file");
    }

    let app_state = Arc::new(AppState {
        project_dir,
        rx: tokio::sync::Mutex::new(dev_server_events_r),
        tx: dev_server_events_s,
        file_tx: file_events_s,
        db: Database::new(),
        dev: Mutex::new(CurrentDevState {
            backend_compiled: true,
            backend_compiling: false,
            backend_restarting: false,
            backend_status: true,
            compiler_messages: vec![],
            vite_status: true,
            features,
            migrations_pending: (false, vec![]),
        }),
    });

    let app = Router::new()
        .route("/", get(|| async {
            // Let user know the server is running
            Html(r###"
                <html style="background-color: #171717; height: 100%; width: 100%;">
                    <head><title>Create Rust App: Development Server</title></head>
                    <body style="text-align: center; color: white; fony-family: sans-serif; padding: 50px;">
                        <img style="margin-right: 12px" height="50px" src="data:image/png;base64,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" alt="" />
                        <h3>Create Rust App: Development Server</h3>
                    </body>
                </html>
            "###)
        }))
        .route("/vitejs-down", get(vitejs_down_handler))
        .route("/vitejs-up", get(vitejs_up_handler))
        .route("/backend-up", get(backend_up_handler))
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    println!("Starting dev server @ http://localhost:{dev_port}/");

    axum::Server::bind(&format!("0.0.0.0:{dev_port}").parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn backend_up_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let sender = state.tx.clone();
    sender.send(DevServerEvent::BackendRestarting(false)).ok();
    sender.send(DevServerEvent::BackendStatus(true)).ok();

    StatusCode::OK.into_response()
}

async fn vitejs_up_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let sender = state.tx.clone();
    sender.send(DevServerEvent::ViteJSStatus(true)).ok();

    StatusCode::OK.into_response()
}

async fn vitejs_down_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let sender = state.tx.clone();
    sender.send(DevServerEvent::ViteJSStatus(true)).ok();

    StatusCode::OK.into_response()
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(stream: WebSocket, state: Arc<AppState>) {
    use axum::extract::ws::Message;
    let (mut sender, mut receiver) = stream.split();

    /*
        SECTION: sending initial state
    */
    let compiler_messages = state.dev.lock().unwrap().compiler_messages.clone();
    let backend_status = state.dev.lock().unwrap().backend_status;
    let backend_compiling = state.dev.lock().unwrap().backend_compiling;
    let backend_restarting = state.dev.lock().unwrap().backend_restarting;
    let backend_compiled = state.dev.lock().unwrap().backend_compiled;
    let vite_status = state.dev.lock().unwrap().vite_status;
    let features = state.dev.lock().unwrap().features.clone();
    let migrations_pending = state.dev.lock().unwrap().migrations_pending.clone();
    sender
        .send(Message::Text(DevServerEvent::FeaturesList(features).json()))
        .await
        .unwrap();
    sender
        .send(Message::Text(
            DevServerEvent::CompileSuccess(backend_compiled).json(),
        ))
        .await
        .unwrap();
    sender
        .send(Message::Text(
            DevServerEvent::CompileMessages(compiler_messages).json(),
        ))
        .await
        .unwrap();
    sender
        .send(Message::Text(
            DevServerEvent::BackendStatus(backend_status).json(),
        ))
        .await
        .unwrap();
    sender
        .send(Message::Text(
            DevServerEvent::BackendRestarting(backend_restarting).json(),
        ))
        .await
        .unwrap();
    sender
        .send(Message::Text(
            DevServerEvent::BackendCompiling(backend_compiling).json(),
        ))
        .await
        .unwrap();
    sender
        .send(Message::Text(
            DevServerEvent::ViteJSStatus(vite_status).json(),
        ))
        .await
        .unwrap();
    sender
        .send(Message::Text(
            DevServerEvent::PendingMigrations(migrations_pending.0, migrations_pending.1).json(),
        ))
        .await
        .unwrap();

    /*
        SECTION: receive dev server events
    */
    let db = state.db.clone();
    let state2 = state.clone();
    let dse_s = state2.tx.clone();
    let mut send_task = tokio::spawn(async move {
        let mut rx = state2.rx.lock().await;

        while let Ok(e) = rx.recv().await {
            let mut send_response = true;

            match e.clone() {
                DevServerEvent::CHECK_MIGRATIONS => {
                    send_response = false;
                    let migrations = controller::get_migrations(&db);
                    if controller::needs_migration(&db) {
                        dse_s
                            .send(DevServerEvent::PendingMigrations(true, migrations))
                            .ok();
                    } else {
                        dse_s
                            .send(DevServerEvent::PendingMigrations(false, migrations))
                            .ok();
                    }
                }
                DevServerEvent::MigrationResponse(success, _) => {
                    // this is a response; we don't need to handle anything on the dev-server side
                    let migrations = controller::get_migrations(&db);
                    dse_s
                        .send(DevServerEvent::PendingMigrations(!success, migrations))
                        .ok();
                }
                DevServerEvent::PendingMigrations(a, b) => {
                    let mut s = state2.dev.lock().unwrap();
                    s.migrations_pending = (a, b);
                }
                DevServerEvent::FeaturesList(list) => {
                    let mut s = state2.dev.lock().unwrap();
                    s.features = list;
                }
                DevServerEvent::BackendCompiling(b) => {
                    let mut s = state2.dev.lock().unwrap();
                    s.backend_compiling = b;
                }
                DevServerEvent::BackendStatus(b) => {
                    let mut s = state2.dev.lock().unwrap();
                    s.backend_status = b;
                }
                DevServerEvent::BackendRestarting(b) => {
                    let mut s = state2.dev.lock().unwrap();
                    s.backend_restarting = b;
                }
                DevServerEvent::CompileSuccess(b) => {
                    let mut s = state2.dev.lock().unwrap();
                    s.backend_compiled = b;
                }
                DevServerEvent::CompileMessages(messages) => {
                    let mut s = state2.dev.lock().unwrap();
                    s.compiler_messages = messages.clone();
                }
                DevServerEvent::SHUTDOWN => {
                    let mut s = state2.dev.lock().unwrap();
                    s.backend_status = false;
                }
                DevServerEvent::ViteJSStatus(b) => {
                    let mut s = state2.dev.lock().unwrap();
                    s.vite_status = b;
                }
            };

            if send_response {
                sender.send(Message::Text(e.json())).await.unwrap();
            }
        }
    });

    /*
        SECTION: receive websocket events
    */
    let state3 = state.clone();
    let file_tx = state.file_tx.clone();
    let mut recv_task = tokio::spawn(async move {
        let state3 = state3.clone();

        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(t) => {
                        let state3 = state3.clone();
                        let file_tx = file_tx.clone();
                        tokio::spawn(async move {
                            let state3 = state3.clone();
                            if t.starts_with("open:") {
                                // HACK: tell the backend server we're about the modify the file (this is a side effect of the `open` crate)
                                //       that way it won't try to recompile as a result of this filesystem modification event

                                let (_, file_name) = t.split_at(5);
                                file_tx.send(file_name.to_string()).ok();

                                // WARNING: this hack causes a race condition between the file above being registered for ignoring
                                //          and the `open` command below which will modify the file
                                //
                                // suggestion 1: change the method by which we open files so they don't get "modified" when opening them
                                //               the new method should be one which can open a specific line and column, unlike the current solution
                                //
                                // suggestion 2: listen for a 'file-registered' event from the backend compiling server
                                //               so we know that it won't re-compile based on the modify event that this
                                //               file opening causes
                                open::that(file_name).unwrap_or_else(|_| {
                                    println!("📝 Could not open file `{file_name}`");
                                });
                            } else if t.eq_ignore_ascii_case("migrate") {
                                let (success, error_message) = controller::migrate_db(&state3.db);

                                state3
                                    .tx
                                    .send(DevServerEvent::MigrationResponse(success, error_message))
                                    .ok();
                            }
                        });
                    }
                    Message::Binary(_) => {}
                    Message::Ping(_) => {}
                    Message::Pong(_) => {}
                    Message::Close(_) => {}
                }
            }
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => {recv_task.abort()},
        _ = (&mut recv_task) => {send_task.abort()},
    };
}
