use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use sqlx::migrate::MigrateDatabase;
use sqlx::Sqlite;
use std::str::FromStr;
use log::{info, error};

// Database constants
const SQLITE_PROTOCOL: &str = "sqlite://";
const DB_PATH: &str = "/Users/tyler/tacc_research/tms_min/tms_min.db";
const POOL_MIN_CONNECTIONS: u32 = 2;
const POOL_MAX_CONNECTIONS: u32 = 12;

pub async fn init_db() -> SqlitePool {
    let url = format!("{}{}", SQLITE_PROTOCOL, DB_PATH);

    // Create database if it doesn't exist
    if !Sqlite::database_exists(&url).await.unwrap_or(false) {
        info!("Creating database {}", &url);
        match Sqlite::create_database(&url).await {
            Ok(_) => info!("Database created successfully"),
            Err(e) => {
                error!("Failed to create database: {}", e);
                panic!("Database creation failed");
            }
        }
    } else {
        info!("Database already exists");
    }

    // Set up connection options
    let options = SqliteConnectOptions::from_str(&url)
        .expect("Invalid DB URL")
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    // Create the connection pool
    let pool = SqlitePoolOptions::new()
        .min_connections(POOL_MIN_CONNECTIONS)
        .max_connections(POOL_MAX_CONNECTIONS)
        .connect_with(options)
        .await
        .expect("Failed to create SQLite pool");

    info!("SQLite connection pool created successfully");

    // Ensure base table exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create 'test' table");

    info!("Verified 'test' table exists");
    pool
}
