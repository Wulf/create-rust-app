use diesel::PgConnection;
use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use once_cell::sync::OnceCell;

#[cfg(feature = "database_postgres")]
type DbCon = diesel::PgConnection;

#[cfg(feature = "database_sqlite")]
type DbCon = diesel::SqliteConnection;

#[cfg(feature = "database_postgres")]
pub type DieselBackend = diesel::pg::Pg;

#[cfg(feature = "database_sqlite")]
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
    pub fn new() -> Database {
        Database {
            pool: Self::get_or_init_pool(),
        }
    }

    /// get a [`Connection`] to a database
    pub fn get_connection(&self) -> Connection {
        self.pool.get().unwrap()
    }

    fn get_or_init_pool() -> &'static Pool {
        #[cfg(debug_assertions)]
        crate::load_env_vars();

        static POOL: OnceCell<Pool> = OnceCell::new();

        POOL.get_or_init(|| {
            Pool::builder()
                .connection_timeout(std::time::Duration::from_secs(5))
                .build(ConnectionManager::<DbCon>::new(Self::database_url()))
                .unwrap()
        })
    }

    fn database_url() -> String {
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable expected.")
    }
}
