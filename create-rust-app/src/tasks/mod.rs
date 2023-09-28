use std::sync::Mutex;
use crate::Database;
use fang::{AsyncQueue, NoTls, Queue};
use once_cell::sync::OnceCell;
// re-export setup for tasks
pub use crate::setup;

/// Returns a reference to the synchronous queue.
///
/// Tasks scheduled in this queue will be executed by the synchronous queue process.
/// A sync queue process can be started by running the `queue` binary.
pub fn queue() -> &'static Queue {
    #[cfg(debug_assertions)]
    crate::load_env_vars();

    static QUEUE: OnceCell<Queue> = OnceCell::new();

    QUEUE.get_or_init(|| {
        let db = Database::new();

        Queue::builder()
            .connection_pool(db.pool.clone())
            .build()
    })
}

/// Returns a reference to the async queue.
///
/// Tasks scheduled in this queue will be executed by the async queue process.
/// An async queue process can be started by running the `async_queue` binary (this is the default for newly generated projects).
///
/// Make sure you connect this queue to your db at the start of your app:
/// ```rust
///    let queue = async_queue();
///    queue.lock().unwrap().connect(NoTls).await.expect("Failed to connect to queue DB");
/// ```
pub fn async_queue() -> &'static Mutex<AsyncQueue<NoTls>> {
    static ASYNC_QUEUE: OnceCell<Mutex<AsyncQueue<NoTls>>> = OnceCell::new();

    ASYNC_QUEUE.get_or_init(|| {
        Mutex::new(create_async_queue(10 /* r2d2's default */))
    })
}

/// Creates a new async queue with the specified max pool size.
/// You should not need to use this function directly.
/// Instead, use `async_queue()` to get a reference to the queue.
///
/// This function is public because it is used by the `async_queue` binary in generated projects.
/// You can also use it to create a separate async queue bin for your own purposes.
/// For example, to support a different async task type.
pub fn create_async_queue(max_pool_size: u32) -> AsyncQueue<NoTls> {
    #[cfg(debug_assertions)]
    crate::load_env_vars();

    AsyncQueue::builder()
        .uri(Database::connection_url())
        .max_pool_size(max_pool_size)
        .build()
}