use fang::Queue;
use once_cell::sync::OnceCell;
use crate::Database;
// re-export setup for tasks
pub use crate::setup;

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
