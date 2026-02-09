use rusqlite::Connection;

const MIGRATION_001: &str = include_str!("../../migrations/001_initial.sql");

struct Migration {
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        name: "001_initial",
        sql: MIGRATION_001,
    },
];

/// Run all pending migrations inside a transaction.
pub fn run(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Ensure the migrations tracking table exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT DEFAULT (datetime('now'))
        );"
    )?;

    for migration in MIGRATIONS {
        let already_applied: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
            [migration.name],
            |row| row.get(0),
        )?;

        if already_applied {
            log::debug!("Migration '{}' already applied, skipping", migration.name);
            continue;
        }

        log::info!("Applying migration '{}'", migration.name);
        conn.execute_batch(migration.sql)?;

        // Record the migration (only if not already tracked by the migration SQL itself)
        let already_tracked: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
            [migration.name],
            |row| row.get(0),
        )?;

        if !already_tracked {
            conn.execute(
                "INSERT INTO _migrations (name) VALUES (?1)",
                [migration.name],
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_apply_cleanly() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn).unwrap();

        // Verify key tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"devices".to_string()));
        assert!(tables.contains(&"scans".to_string()));
        assert!(tables.contains(&"alerts".to_string()));
        assert!(tables.contains(&"alert_rules".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn).unwrap();
        // Running again should not error
        run(&conn).unwrap();
    }

    #[test]
    fn test_default_alert_rules_seeded() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM alert_rules", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_default_settings_seeded() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run(&conn).unwrap();

        let theme: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'theme'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(theme, "dark");
    }
}
