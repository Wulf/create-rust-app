///
/// This binary runs the "common" tasks queue
///
/// Remember to add your imports (mod statements) here as well,
/// otherwise your `tasks/*.rs` may not compile either since this binary (`queue.rs`)
/// will have a different set of included modules compared to `main.rs`
///
/// Use `cargo run --bin queue` in development
/// Use `cargo run --bin queue --release` in production
///

extern crate diesel;

use std::thread::sleep;
use std::time::Duration;

use fang::{RetentionMode, WorkerPool};
use fang::Queue;

mod schema;
mod models;
mod tasks;

const NUM_WORKERS: u32 = 2;

/// Executes tasks in the default work queue
pub fn main() {
    let queue = create_rust_app::tasks::queue();

    println!("Starting pool for 'sync' tasks...");

    let mut worker_pool = WorkerPool::<Queue>::builder()
        .queue(queue.clone())
        .retention_mode(RetentionMode::RemoveFinished)
        .task_type("sync".to_string())
        .number_of_workers(NUM_WORKERS)
        .build();

    worker_pool.start().unwrap();

    loop {
        // fang doesn't expose a way to join with the worker threads, so we'll
        // make the main thread sleep as the worker threads run
        sleep(Duration::from_secs(24 * 60));
    }
}