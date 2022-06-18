use diesel::{sql_query, sql_types::Text, query_dsl::RunQueryDsl};
use diesel_migrations::{any_pending_migrations, run_pending_migrations};
use serde::{Deserialize, Serialize};
use crate::auth::{Auth, Role};
use crate::Database;

#[derive(Debug, Deserialize, QueryableByName)]
pub struct MyQueryResult {
    #[sql_type = "Text"]
    pub json: String,
}

#[derive(Serialize, Deserialize)]
pub struct MySqlQuery {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub message: String,
}

/// /db/query
pub fn query_db(db: &Database, body: &MySqlQuery) -> Result<String, ()> {
    let q = format!("SELECT json_agg(q) as json FROM ({}) q;", body.query);
    let db = db.pool.get().unwrap();

    let rows = sql_query(q.as_str()).get_result::<MyQueryResult>(&db);
    if rows.is_err() {
        return Err(());
    }

    let result = rows.unwrap().json;

    Ok(result)
}

/// /auth/has-system-role
pub fn check_system_role(auth: &Auth) -> bool {
    auth.has_permission("system".to_string())
}

/// /auth/add-system-role
pub fn add_system_role(db: &Database, auth: &Auth) -> bool {
    let db = db.pool.clone().get().unwrap();

    Role::assign(&db, auth.user_id, "system").unwrap()
}

/// /db/is-connected
pub fn is_connected(db: &Database) -> bool {
    let db = db.pool.clone().get().unwrap();
    let is_connected = sql_query("SELECT 1;").execute(&db);
    is_connected.is_err()
}

/// /db/needs-migration
pub fn needs_migration(db: &Database) -> bool {
    let db = db.pool.clone().get().unwrap();
    // This will run the necessary migrations.
    let has_migrations_pending = any_pending_migrations(&db).unwrap();
    has_migrations_pending
}

/// /db/migrate
pub fn migrate_db(db: &Database) -> bool {
    let db = db.pool.clone().get().unwrap();

    // This will run the necessary migrations.
    let has_migrations_pending = any_pending_migrations(&db).unwrap();

    if !has_migrations_pending {
        return true;
    }

    let op = run_pending_migrations(&db);

    if op.is_err() {
        println!("{:#?}", op.err());
        return false;
    }

    true
}

/// /health
pub fn health() -> () {
    ()
}