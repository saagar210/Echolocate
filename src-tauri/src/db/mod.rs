pub mod cache;
pub mod encryption;
pub mod migrations;
pub mod queries;
pub mod recovery;

use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

/// Initialize the SQLite database with connection pooling and WAL mode.
pub fn init_db(app_data_dir: &Path) -> Result<Pool<SqliteConnectionManager>, Box<dyn std::error::Error>> {
    let db_path = app_data_dir.join("echolocate.db");
    log::info!("Database path: {}", db_path.display());

    let manager = SqliteConnectionManager::file(&db_path)
        .with_init(|conn| {
            // Enable WAL mode for better concurrent read performance
            conn.execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA foreign_keys=ON;
                 PRAGMA busy_timeout=5000;"
            )?;
            Ok(())
        });

    let pool = Pool::builder()
        .max_size(4)
        .build(manager)?;

    // Run migrations
    let conn = pool.get()?;
    migrations::run(&conn)?;

    log::info!("Database initialized successfully");
    Ok(pool)
}

#[cfg(test)]
pub fn init_test_db() -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::memory()
        .with_init(|conn| {
            conn.execute_batch("PRAGMA foreign_keys=ON;")?;
            Ok(())
        });
    let pool = Pool::builder().max_size(1).build(manager).unwrap();
    let conn = pool.get().unwrap();
    migrations::run(&conn).unwrap();
    pool
}
