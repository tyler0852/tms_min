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
    let tables = ["table_a", "table_b", "table_c"];

    // Ensure tables exist (same as before)
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

    for table in &tables {
        let mut tx = pool.0.begin().await.unwrap();

        // INSERT #1
        let insert_query = format!("INSERT INTO {} (value) VALUES (?)", table);
        sqlx::query(&insert_query)
            .bind(format!("data {}", rand::thread_rng().gen_range(0..100000)))
            .execute(&mut *tx)
            .await
            .unwrap();

        // READ inside txn
        let count_query = format!("SELECT COUNT(*) FROM {}", table);
        sqlx::query(&count_query)
            .fetch_one(&mut *tx)
            .await
            .unwrap();

        // INSERT #2 (extra write load)
        sqlx::query(&insert_query)
            .bind(format!("extra {}", rand::thread_rng().gen_range(0..100000)))
            .execute(&mut *tx)
            .await
            .unwrap();

        // Perform a tiny file read (simulates PEM checks)
        let _ = std::fs::read("/Users/tyler/tacc_research/tms_min/tms_min.db");

        tx.commit().await.unwrap();

        // Small random delay
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
        // Small delay (as before)
        let delay = rand::thread_rng().gen_range(5..20);
        sleep(Duration::from_millis(delay)).await;

        // SELECT rows
        let limit = rand::thread_rng().gen_range(5..15);
        let query = format!("SELECT * FROM {} LIMIT {}", table, limit);
        let rows = sqlx::query(&query)
            .fetch_all(pool.0)
            .await
            .unwrap();

        total_rows += rows.len();

        // Extra COUNT query (increases load)
        let count_query = format!("SELECT COUNT(*) FROM {}", table);
        let _ = sqlx::query(&count_query)
            .fetch_one(pool.0)
            .await
            .unwrap();

        // Tiny file read (simulated filesystem pressure)
        let _ = std::fs::read("/Users/tyler/tacc_research/tms_min/tms_min.db");
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
