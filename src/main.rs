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
    
    println!("Server running at http://localhost:3001");
    Server::new(TcpListener::bind("0.0.0.0:3001"))
        .run(app)
        .await
}