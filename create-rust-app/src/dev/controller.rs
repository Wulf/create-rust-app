use crate::Database;
use diesel::{
    migration::{Migration, MigrationSource},
    query_dsl::RunQueryDsl,
    sql_query,
    sql_types::Text,
};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use serde::{Deserialize, Serialize};

use super::{CreateRustAppMigration, MigrationStatus};

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
///
/// # Errors
/// * query fails
///
/// # Panics
/// * cannot connect to the database
pub fn query_db(db: &Database, body: &MySqlQuery) -> Result<String, diesel::result::Error> {
    let q = format!("SELECT json_agg(q) as json FROM ({}) q;", body.query);
    let mut db = db.pool.get().unwrap();

    Ok(sql_query(q.as_str())
        .get_result::<MyQueryResult>(&mut db)?
        .json)
}

/// /db/is-connected
///
/// # Panics
/// * cannot connect to the database
#[must_use]
pub fn is_connected(db: &Database) -> bool {
    let mut db = db.pool.clone().get().unwrap();
    let is_connected = sql_query("SELECT 1;").execute(&mut db);
    is_connected.is_err()
}

/// # Panics
/// * cannot connect to the database
/// * cannot find the migrations directory
#[must_use]
pub fn get_migrations(db: &Database) -> Vec<CreateRustAppMigration> {
    // Vec<diesel::migration::Migration> {
    let mut db = db.pool.clone().get().unwrap();

    let source = FileBasedMigrations::find_migrations_directory().unwrap();

    #[cfg(feature = "database_sqlite")]
    let file_migrations =
        MigrationSource::<crate::database::DieselBackend>::migrations(&source).unwrap();
    #[cfg(feature = "database_postgres")]
    let file_migrations =
        MigrationSource::<crate::database::DieselBackend>::migrations(&source).unwrap();

    let db_migrations = MigrationHarness::applied_migrations(&mut db).unwrap();
    let pending_migrations = MigrationHarness::pending_migrations(&mut db, source).unwrap();

    let mut all_migrations = vec![];

    for fm in &file_migrations {
        all_migrations.push(CreateRustAppMigration {
            name: fm.name().to_string(),
            version: fm.name().version().to_string(),
            status: MigrationStatus::Unknown,
        });
    }

    // update the status for any pending file_migrations
    for pm in &pending_migrations {
        if let Some(existing) = all_migrations.iter_mut().find(|m| {
            m.version
                .eq_ignore_ascii_case(&pm.name().version().to_string())
        }) {
            existing.status = MigrationStatus::Pending;
        }
    }

    for dm in &db_migrations {
        match all_migrations
            .iter_mut()
            .find(|m| m.version.eq_ignore_ascii_case(&dm.to_string()))
        {
            Some(existing) => {
                existing.status = MigrationStatus::Applied;
            }
            None => all_migrations.push(CreateRustAppMigration {
                name: format!("{dm}_?"),
                version: dm.to_string(),
                status: MigrationStatus::AppliedButMissingLocally,
            }),
        }
    }

    all_migrations
}

/// /db/needs-migration
/// checks if a migration is needed
///
/// # Panics
/// * cannot connect to the database
/// * cannot find the migrations directory
///
/// TODO: return a Result instead of panicking
#[must_use]
pub fn needs_migration(db: &Database) -> bool {
    let mut db = db.pool.clone().get().unwrap();

    let source = FileBasedMigrations::find_migrations_directory().unwrap();
    MigrationHarness::has_pending_migration(&mut db, source).unwrap()
}

/// /db/migrate
/// performs any pending migrations
///
/// # Panics
/// * cannot connect to the database
/// * cannot find the migrations directory
/// * cannot run the migrations
///
/// TODO: return a Result instead of a tuple (bool, Option<String>), this is Rust, not Go
#[must_use]
pub fn migrate_db(db: &Database) -> (bool, /* error message: */ Option<String>) {
    let mut db = db.pool.clone().get().unwrap();

    let source = FileBasedMigrations::find_migrations_directory().unwrap();
    let has_pending_migrations =
        MigrationHarness::has_pending_migration(&mut db, source.clone()).unwrap();

    if !has_pending_migrations {
        return (true, None);
    }

    let op = MigrationHarness::run_pending_migrations(&mut db, source);
    match op {
        Ok(_) => (true, None),
        Err(err) => {
            println!("{err:#?}");
            (false, Some(err.to_string()))
        }
    }
}

/// /health
pub const fn health() {}
