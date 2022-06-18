use diesel::{
    r2d2::{self, ConnectionManager, PooledConnection},
    PgConnection,
};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type Connection = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool,
}

impl Database {
    pub fn new() -> Database {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable expected.");
        let database_pool = Pool::builder()
            .connection_timeout(std::time::Duration::from_secs(5))
            .build(ConnectionManager::<PgConnection>::new(database_url))
            .unwrap();

        Database {
            pool: database_pool,
        }
    }
}
