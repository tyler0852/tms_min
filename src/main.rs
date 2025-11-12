use poem::{get, post, handler, listener::TcpListener, web::Data, Route, Server};
use sqlx::sqlite::SqlitePool;

///// BASELINE ENPOINT /////
#[handler]
async fn baseline(pool: Data<&SqlitePool>) -> String {
    "Server is running".to_string()
}

///// WRITE HEAVY ENDPOINT /////
#[handler]
async fn writeheavy(pool: Data<&SqlitePool>) -> String {
    // Create the table if it doesn't exist
    sqlx::query("CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY AUTOINCREMENT, value TEXT)")
        .execute(pool.0)
        .await
        .unwrap();

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

///// READ HEAVY /////
#[handler]
async fn readheavy(pool: Data<&SqlitePool>) -> String {
    sqlx::query("CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY AUTOINCREMENT, value TEXT)")
        .execute(pool.0)
        .await
        .unwrap();

    // Read roughly one writeheavy worth of data
    let rows = sqlx::query("SELECT * FROM test LIMIT 10")
        .fetch_all(pool.0)
        .await
        .unwrap();

    format!("Read-heavy operation complete. Rows read: {}", rows.len())
}


///// MAIN /////
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Create a SQLite database connection pool
    let database_url = "sqlite:///Users/tyler/tacc_research/tms_min/tms_min.db";
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to connect to database");

    use poem::EndpointExt;    
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