///////////////////////////////////////////////////
//////////////////// VERSION 1 ////////////////////
///////////////////////////////////////////////////

use poem::{get, post, handler, listener::TcpListener, web::Data, Route, Server, EndpointExt};
use sqlx::sqlite::SqlitePool;

mod db_init;
use db_init::init_db;

////////////////////////////
///// BASELINE ENPOINT /////
////////////////////////////
#[handler]
async fn baseline() -> String {
    "Server is running".to_string()
}


////////////////////////////////
///// WRITE HEAVY ENDPOINT /////
////////////////////////////////
#[handler]
async fn writeheavy(pool: Data<&SqlitePool>) -> String {
    // Insert multiple rows in a loop
    for i in 0..10 {
        sqlx::query("INSERT INTO test (value) VALUES (?)")
            .bind(format!("row {}", i))
            .execute(pool.0)
            .await
            .unwrap();
    }

    "Write-heavy operation complete".to_string()
}


//////////////////////
///// READ HEAVY /////
//////////////////////
#[handler]
async fn readheavy(pool: Data<&SqlitePool>) -> String {
    // Read roughly one writeheavy worth of data
    let rows = sqlx::query("SELECT * FROM test LIMIT 10")
        .fetch_all(pool.0)
        .await
        .unwrap();

    format!("Read-heavy operation complete. Rows read: {}", rows.len())
}


////////////////
///// MAIN /////
////////////////
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Create a SQLite database connection pool
    let pool = init_db().await;

    // Establish routes
    let app = Route::new()
        .at("/baseline", get(baseline))
        .at("/writeheavy", post(writeheavy))
        .at("/readheavy", get(readheavy))
        .data(pool);  // Make database available to handlers
    
    println!("Server running at http://localhost:3000");
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}


///////////////////////////////////////////////////
//////////////////// VERSION 2 ////////////////////
///////////////////////////////////////////////////

use poem::{get, post, handler, listener::TcpListener, web::Data, Route, Server, EndpointExt};
use sqlx::sqlite::SqlitePool;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

mod db_init;
use db_init::init_db;

////////////////////////////
///// BASELINE ENDPOINT ////
////////////////////////////
#[handler]
async fn baseline() -> String {
    "Server is running".to_string()
}

////////////////////////////////
///// WRITE HEAVY ENDPOINT /////
////////////////////////////////
#[handler]
async fn writeheavy(pool: Data<&SqlitePool>) -> String {
    // Define multiple generic tables
    let tables = ["table_a", "table_b", "table_c"];

    // Ensure tables exist
    for table in &tables {
        let create_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, value TEXT)",
            table
        );
        sqlx::query(&create_query)
            .execute(pool.0)
            .await
            .unwrap();
    }

    // Each request will perform several inserts across multiple tables
    for table in &tables {
        // Start a new transaction for each insert to increase lock contention
        let mut tx = pool.0.begin().await.unwrap();

        let insert_query = format!("INSERT INTO {} (value) VALUES (?)", table);
        sqlx::query(&insert_query)
            .bind(format!("data {}", rand::thread_rng().gen_range(0..100000)))
            .execute(&mut *tx)
            .await
            .unwrap();

        tx.commit().await.unwrap();

        // Add a small random delay to force overlap between concurrent requests
        let delay = rand::thread_rng().gen_range(10..50);
        sleep(Duration::from_millis(delay)).await;
    }

    "Write-heavy operation complete".to_string()
}

//////////////////////
///// READ HEAVY /////
//////////////////////
#[handler]
async fn readheavy(pool: Data<&SqlitePool>) -> String {
    let tables = ["table_a", "table_b", "table_c"];
    let mut total_rows = 0;

    for table in &tables {
        // Random small delay between reads to simulate slow I/O
        let delay = rand::thread_rng().gen_range(5..20);
        sleep(Duration::from_millis(delay)).await;

        // Read a random number of rows from each table
        let limit = rand::thread_rng().gen_range(5..15);
        let query = format!("SELECT * FROM {} LIMIT {}", table, limit);

        let rows = sqlx::query(&query)
            .fetch_all(pool.0)
            .await
            .unwrap();

        total_rows += rows.len();
    }

    format!("Read-heavy operation complete. Total rows read: {}", total_rows)
}

////////////////
///// MAIN /////
////////////////
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let pool = init_db().await;

    let app = Route::new()
        .at("/baseline", get(baseline))
        .at("/writeheavy", post(writeheavy))
        .at("/readheavy", get(readheavy))
        .data(pool);

    println!("Server running at http://localhost:3000");
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}