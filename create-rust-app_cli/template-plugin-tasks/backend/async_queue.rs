///
/// This binary runs the "common" tasks queue
///
/// Remember to add your imports (mod statements) here as well,
/// otherwise your `tasks/*.rs` may not compile either since this binary (`queue.rs`)
/// will have a different set of included modules compared to `main.rs`
///
/// Use `cargo run --bin async_queue` in development
/// Use `cargo run --bin async_queue --release` in production
///

extern crate diesel;

use std::thread;
use std::time::Duration;
use create_rust_app::Database;
use fang::{AsyncQueue, NoTls, RetentionMode};
use fang::asynk::async_worker_pool::AsyncWorkerPool;

mod schema;
mod models;
mod tasks;

const NUM_WORKERS: u32 = 2;

/// Executes tasks in the default work queue
#[tokio::main]
pub async fn main() {
    println!("Starting async pool for 'async' tasks...");

    let mut async_queue = create_rust_app::tasks::create_async_queue(NUM_WORKERS);

    async_queue.connect(NoTls).await.expect("Failed to connect to async queue database");

    let mut pool: AsyncWorkerPool<AsyncQueue<NoTls>> = AsyncWorkerPool::builder()
        .number_of_workers(NUM_WORKERS)
        .queue(async_queue)
        .retention_mode(RetentionMode::RemoveFinished)
        // if you want to run tasks of the specific kind
        .task_type("async".to_string())
        .build();

    pool.start().await;

    loop {
        thread::sleep(Duration::from_secs(24 * 60));
    }
}
