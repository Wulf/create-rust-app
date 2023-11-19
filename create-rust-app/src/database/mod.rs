use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use once_cell::sync::OnceCell;

#[cfg(feature = "database_postgres")]
type DbCon = diesel::PgConnection;

#[cfg(feature = "database_sqlite")]
type DbCon = diesel::SqliteConnection;

#[cfg(all(feature = "database_postgres", debug_assertions))]
#[allow(dead_code)]
pub type DieselBackend = diesel::pg::Pg;

#[cfg(all(feature = "database_sqlite", debug_assertions))]
#[allow(dead_code)]
pub type DieselBackend = diesel::sqlite::Sqlite;

pub type Pool = r2d2::Pool<ConnectionManager<DbCon>>;
pub type Connection = PooledConnection<ConnectionManager<DbCon>>;

#[derive(Clone)]
/// wrapper function for a database pool
pub struct Database {
    pub pool: &'static Pool,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    /// create a new [`Database`]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pool: Self::get_or_init_pool(),
        }
    }

    /// get a [`Connection`] to a database
    ///
    /// TODO: return a `Result` and let the caller handle the error (a server might want to return a 500 error if the database is not available instead of crashing)
    ///
    /// # Panics
    /// * if the database is not available
    #[must_use]
    pub fn get_connection(&self) -> Connection {
        self.pool.get().unwrap()
    }

    fn get_or_init_pool() -> &'static Pool {
        static POOL: OnceCell<Pool> = OnceCell::new();
        #[cfg(debug_assertions)]
        crate::load_env_vars();

        POOL.get_or_init(|| {
            Pool::builder()
                .connection_timeout(std::time::Duration::from_secs(5))
                .build(ConnectionManager::<DbCon>::new(Self::connection_url()))
                .unwrap()
        })
    }

    /// get the connection url for the database
    ///
    /// # Panics
    /// * if the `DATABASE_URL` environment variable is not set
    #[must_use]
    pub fn connection_url() -> String {
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable expected.")
    }
}
