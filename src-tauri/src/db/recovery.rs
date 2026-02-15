/// Database recovery and resilience mechanisms
/// Handles corruption, cleanup, and disaster recovery

use rusqlite::Connection;
use std::path::Path;
use std::fs;

/// Check database integrity
pub fn check_integrity(conn: &Connection) -> Result<bool, String> {
    match conn.query_row(
        "PRAGMA integrity_check;",
        [],
        |row| row.get::<_, String>(0),
    ) {
        Ok(result) => {
            if result == "ok" {
                Ok(true)
            } else {
                log::error!("Integrity check failed: {}", result);
                Ok(false)
            }
        }
        Err(e) => {
            log::error!("Integrity check error: {}", e);
            Err(format!("Integrity check failed: {}", e))
        }
    }
}

/// Optimize database (VACUUM and ANALYZE)
pub fn optimize(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("VACUUM; ANALYZE;")
        .map_err(|e| format!("Optimization failed: {}", e))
}

/// Create automatic backup before maintenance
pub fn create_backup(db_path: &Path) -> Result<std::path::PathBuf, String> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = db_path.with_file_name(format!(
        "echolocate_backup_{}.db",
        timestamp
    ));

    fs::copy(db_path, &backup_path)
        .map_err(|e| format!("Backup creation failed: {}", e))?;

    log::info!("Database backed up to: {}", backup_path.display());
    Ok(backup_path)
}

/// Restore from backup
pub fn restore_from_backup(backup_path: &Path, db_path: &Path) -> Result<(), String> {
    fs::copy(backup_path, db_path)
        .map_err(|e| format!("Restore failed: {}", e))?;

    log::info!("Database restored from: {}", backup_path.display());
    Ok(())
}

/// Clean up old backups (keep last N)
pub fn cleanup_old_backups(db_dir: &Path, keep_count: usize) -> Result<(), String> {
    let mut backups = Vec::new();

    for entry in fs::read_dir(db_dir)
        .map_err(|e| format!("Read dir failed: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Read entry failed: {}", e))?;
        let path = entry.path();

        if let Some(filename) = path.file_name() {
            if let Some(filename_str) = filename.to_str() {
                if filename_str.starts_with("echolocate_backup_") && filename_str.ends_with(".db") {
                    backups.push(path);
                }
            }
        }
    }

    // Sort by modification time, keep newest
    backups.sort_by_key(|path| {
        fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    // Remove old backups
    while backups.len() > keep_count {
        let old = backups.remove(0);
        fs::remove_file(&old)
            .map_err(|e| format!("Failed to remove old backup: {}", e))?;
        log::info!("Removed old backup: {}", old.display());
    }

    Ok(())
}

/// Repair corrupted database by rebuilding
pub fn repair_database(conn: &Connection, db_path: &Path) -> Result<(), String> {
    // 1. Check integrity
    if !check_integrity(conn)? {
        log::warn!("Database integrity check failed, attempting repair...");

        // 2. Create backup
        drop(conn); // Close connection before backing up
        let backup_path = create_backup(db_path)?;
        log::info!("Created backup at: {}", backup_path.display());

        // 3. Try VACUUM and REINDEX
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        conn.execute_batch("REINDEX;")
            .map_err(|e| {
                log::error!("REINDEX failed: {}", e);
                format!("REINDEX failed: {}", e)
            })?;

        conn.execute_batch("VACUUM;")
            .map_err(|e| {
                log::error!("VACUUM failed: {}", e);
                format!("VACUUM failed: {}", e)
            })?;

        // 4. Verify repair
        if check_integrity(&conn)? {
            log::info!("Database repair successful");
            Ok(())
        } else {
            log::error!("Database repair failed");
            Err("Database repair failed - restore from backup".to_string())
        }
    } else {
        log::info!("Database integrity check passed");
        Ok(())
    }
}

/// Detect and handle common database issues
pub fn diagnose_and_repair(conn: &Connection, db_path: &Path) -> Result<(), String> {
    // Check integrity
    match check_integrity(conn) {
        Ok(true) => {
            log::info!("Database is healthy");

            // Optimize if healthy
            optimize(conn)?;
            Ok(())
        }
        Ok(false) => {
            log::warn!("Database integrity issue detected");
            repair_database(conn, db_path)
        }
        Err(e) => {
            log::error!("Integrity check error: {}", e);
            repair_database(conn, db_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn test_check_integrity_healthy_db() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        assert!(check_integrity(&conn).unwrap());
    }

    #[test]
    fn test_optimize_database() {
        let pool = db::init_test_db();
        let conn = pool.get().unwrap();

        assert!(optimize(&conn).is_ok());
    }

    #[test]
    fn test_backup_creation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create a test database
        Connection::open(&db_path).unwrap();

        let backup_result = create_backup(&db_path);
        assert!(backup_result.is_ok());

        let backup_path = backup_result.unwrap();
        assert!(backup_path.exists());
    }

    #[test]
    fn test_backup_restore() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let restore_path = temp_dir.path().join("restored.db");

        // Create original
        Connection::open(&db_path).unwrap();

        // Backup
        let backup_path = create_backup(&db_path).unwrap();

        // Restore to different location
        assert!(restore_from_backup(&backup_path, &restore_path).is_ok());
        assert!(restore_path.exists());
    }
}
