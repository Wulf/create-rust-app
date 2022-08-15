use diesel::{sql_query, sql_types::Text, query_dsl::RunQueryDsl};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use serde::{Deserialize, Serialize};
use crate::auth::{Auth, Role};
use crate::Database;

#[derive(Debug, Deserialize, QueryableByName)]
pub struct MyQueryResult {
    #[diesel(sql_type=Text)]
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
    let mut db = db.pool.get().unwrap();

    let rows = sql_query(q.as_str()).get_result::<MyQueryResult>(&mut db);
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
    let mut db = db.pool.clone().get().unwrap();

    Role::assign(&mut db, auth.user_id, "system").unwrap()
}

/// /db/is-connected
pub fn is_connected(db: &Database) -> bool {
    let mut db = db.pool.clone().get().unwrap();
    let is_connected = sql_query("SELECT 1;").execute(&mut db);
    is_connected.is_err()
}

/// /db/needs-migration
/// checks if a migration is needed
pub fn needs_migration(db: &Database) -> bool {
    let mut db = db.pool.clone().get().unwrap();
    
    let source = FileBasedMigrations::find_migrations_directory().unwrap();
    MigrationHarness::has_pending_migration(&mut db, source).unwrap()
}

/// /db/migrate
/// performs any pending migrations
pub fn migrate_db(db: &Database) -> bool {
    let mut db = db.pool.clone().get().unwrap();

    let source = FileBasedMigrations::find_migrations_directory().unwrap();
    let has_pending_migrations = MigrationHarness::has_pending_migration(&mut db, source.clone()).unwrap();

    if !has_pending_migrations {
        return true;
    }

    let op = MigrationHarness::run_pending_migrations(&mut db, source);

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