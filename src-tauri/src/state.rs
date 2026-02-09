use std::sync::Mutex;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::network::oui::OuiDatabase;

/// Shared application state managed by Tauri.
/// Accessed via `tauri::State<AppState>` in commands.
pub struct AppState {
    pub db: Pool<SqliteConnectionManager>,
    pub oui_db: OuiDatabase,
    /// Handle to the background monitoring task, if active.
    pub monitor_handle: Mutex<Option<JoinHandle<()>>>,
    /// Token to cancel the monitoring loop.
    pub monitor_cancel: Mutex<Option<CancellationToken>>,
    /// Token to cancel the current in-progress scan.
    pub scan_cancel: Mutex<Option<CancellationToken>>,
}

impl AppState {
    pub fn new(db: Pool<SqliteConnectionManager>, oui_db: OuiDatabase) -> Self {
        Self {
            db,
            oui_db,
            monitor_handle: Mutex::new(None),
            monitor_cancel: Mutex::new(None),
            scan_cancel: Mutex::new(None),
        }
    }

    /// Get a database connection from the pool.
    pub fn conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, r2d2::Error> {
        self.db.get()
    }
}
