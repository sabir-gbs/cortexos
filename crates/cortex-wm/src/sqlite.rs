//! SQLite-backed window manager implementation.

use cortex_core::{AppInstanceId, UserId};
use cortex_db::error::DbError;

use crate::error::{Result, WmError};
use crate::service::WindowManagerService;
use crate::types::*;

/// SQLite-backed window manager.
#[derive(Clone)]
pub struct SqliteWindowManager {
    pool: cortex_db::Pool,
}

impl SqliteWindowManager {
    /// Create a new window manager backed by the given pool.
    pub fn new(pool: cortex_db::Pool) -> Self {
        Self { pool }
    }

    /// Initialize the schema (run once at startup).
    pub fn init_schema(&self) -> Result<()> {
        self.pool.write(|conn| {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS workspaces (
                    id TEXT PRIMARY KEY,
                    user_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    idx INTEGER NOT NULL,
                    active INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE INDEX IF NOT EXISTS idx_workspaces_user ON workspaces(user_id);

                CREATE TABLE IF NOT EXISTS windows (
                    id TEXT PRIMARY KEY,
                    instance_id TEXT NOT NULL,
                    user_id TEXT NOT NULL,
                    workspace_id TEXT NOT NULL,
                    title TEXT NOT NULL DEFAULT '',
                    state TEXT NOT NULL DEFAULT 'normal',
                    x INTEGER NOT NULL DEFAULT 0,
                    y INTEGER NOT NULL DEFAULT 0,
                    width INTEGER NOT NULL DEFAULT 800,
                    height INTEGER NOT NULL DEFAULT 600,
                    z_index INTEGER NOT NULL DEFAULT 0,
                    focused INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
                );
                CREATE INDEX IF NOT EXISTS idx_windows_user_ws ON windows(user_id, workspace_id);
                CREATE INDEX IF NOT EXISTS idx_windows_workspace ON windows(workspace_id);
                ",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
            Ok(())
        })?;
        Ok(())
    }

    fn row_to_window(row: &rusqlite::Row) -> std::result::Result<Window, rusqlite::Error> {
        let state_str: String = row.get(5)?;
        let state = match state_str.as_str() {
            "normal" => WindowState::Normal,
            "minimized" => WindowState::Minimized,
            "maximized" => WindowState::Maximized,
            "closed" => WindowState::Closed,
            _ => WindowState::Normal,
        };
        let focused: i32 = row.get(10)?;

        let instance_id_str: String = row.get(1)?;
        let instance_id = uuid::Uuid::parse_str(&instance_id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid instance_id uuid: {e}"),
                )),
            )
        })?;

        let user_id_str: String = row.get(2)?;
        let user_id = uuid::Uuid::parse_str(&user_id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid user_id uuid: {e}"),
                )),
            )
        })?;

        Ok(Window {
            id: WindowId(row.get(0)?),
            instance_id: AppInstanceId(instance_id),
            user_id: UserId(user_id),
            workspace_id: WorkspaceId(row.get(3)?),
            title: row.get(4)?,
            state,
            x: row.get(6)?,
            y: row.get(7)?,
            width: row.get(8)?,
            height: row.get(9)?,
            z_index: row.get(11)?,
            focused: focused != 0,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }

    fn row_to_workspace(row: &rusqlite::Row) -> std::result::Result<Workspace, rusqlite::Error> {
        let active: i32 = row.get(4)?;

        let user_id_str: String = row.get(1)?;
        let user_id = uuid::Uuid::parse_str(&user_id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid user_id uuid: {e}"),
                )),
            )
        })?;

        Ok(Workspace {
            id: WorkspaceId(row.get(0)?),
            user_id: UserId(user_id),
            name: row.get(2)?,
            index: row.get(3)?,
            active: active != 0,
            created_at: row.get(5)?,
        })
    }

    const WINDOW_COLS: &'static str = "id, instance_id, user_id, workspace_id, title, state, x, y, width, height, focused, z_index, created_at, updated_at";
}

impl WindowManagerService for SqliteWindowManager {
    async fn open_window(&self, user_id: &UserId, req: OpenWindowRequest) -> Result<Window> {
        let window_id = WindowId::new();
        let now = chrono::Utc::now().to_rfc3339();
        let ws_id = match &req.workspace_id {
            Some(id) => WorkspaceId(id.clone()),
            None => {
                let ws = self.get_active_workspace(user_id).await?;
                ws.id
            }
        };
        let uid_str = user_id.0.to_string();
        let iid_str = req.instance_id.clone();
        let wid_str = window_id.0.clone();
        let wsid_str = ws_id.0.clone();
        let title = req.title.clone();

        let result = self.pool.write(|conn| {
            let max_z: i64 = conn
                .query_row(
                    "SELECT COALESCE(MAX(z_index), 0) FROM windows WHERE workspace_id = ?1 AND state != 'closed'",
                    [&wsid_str],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            conn.execute(
                "UPDATE windows SET focused = 0, updated_at = ?1 WHERE user_id = ?2 AND workspace_id = ?3 AND state != 'closed'",
                rusqlite::params![now, uid_str, wsid_str],
            ).map_err(|e| DbError::Query(e.to_string()))?;

            conn.execute(
                "INSERT INTO windows (id, instance_id, user_id, workspace_id, title, state, x, y, width, height, z_index, focused, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'normal', ?6, ?7, ?8, ?9, ?10, 1, ?11, ?11)",
                rusqlite::params![wid_str, iid_str, uid_str, wsid_str, title, req.x, req.y, req.width, req.height, max_z + 1, now],
            ).map_err(|e| DbError::Query(e.to_string()))?;

            Ok(max_z + 1)
        }).map_err(WmError::Db)?;

        let instance_uuid = uuid::Uuid::parse_str(&iid_str)
            .map_err(|e| WmError::InvalidOperation(format!("invalid instance_id uuid: {e}")))?;

        Ok(Window {
            id: window_id,
            instance_id: AppInstanceId(instance_uuid),
            user_id: user_id.clone(),
            workspace_id: ws_id,
            title: req.title,
            state: WindowState::Normal,
            x: req.x,
            y: req.y,
            width: req.width,
            height: req.height,
            z_index: result as u32,
            focused: true,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    async fn close_window(&self, window_id: &str) -> Result<()> {
        let rows = self.pool.write(|conn| {
            conn.execute(
                "UPDATE windows SET state = 'closed', focused = 0, updated_at = datetime('now') WHERE id = ?1",
                [window_id],
            ).map_err(|e| DbError::Query(e.to_string()))
        }).map_err(WmError::Db)?;

        if rows == 0 {
            return Err(WmError::WindowNotFound(window_id.to_string()));
        }
        Ok(())
    }

    async fn minimize_window(&self, window_id: &str) -> Result<Window> {
        self.update_state(window_id, "minimized")
    }

    async fn maximize_window(&self, window_id: &str) -> Result<Window> {
        self.update_state(window_id, "maximized")
    }

    async fn restore_window(&self, window_id: &str) -> Result<Window> {
        self.update_state(window_id, "normal")
    }

    async fn move_window(&self, window_id: &str, req: MoveWindowRequest) -> Result<Window> {
        let rows = self.pool.write(|conn| {
            conn.execute(
                "UPDATE windows SET x = ?1, y = ?2, updated_at = datetime('now') WHERE id = ?3 AND state != 'closed'",
                rusqlite::params![req.x, req.y, window_id],
            ).map_err(|e| DbError::Query(e.to_string()))
        }).map_err(WmError::Db)?;

        if rows == 0 {
            return Err(WmError::WindowNotFound(window_id.to_string()));
        }
        self.get_window(window_id).await
    }

    async fn resize_window(&self, window_id: &str, req: ResizeWindowRequest) -> Result<Window> {
        let rows = self.pool.write(|conn| {
            conn.execute(
                "UPDATE windows SET width = ?1, height = ?2, updated_at = datetime('now') WHERE id = ?3 AND state != 'closed'",
                rusqlite::params![req.width, req.height, window_id],
            ).map_err(|e| DbError::Query(e.to_string()))
        }).map_err(WmError::Db)?;

        if rows == 0 {
            return Err(WmError::WindowNotFound(window_id.to_string()));
        }
        self.get_window(window_id).await
    }

    async fn focus_window(&self, window_id: &str) -> Result<Window> {
        let win = self.get_window(window_id).await?;
        let uid_str = win.user_id.0.to_string();
        let wsid_str = win.workspace_id.0.clone();
        let wid_str = window_id.to_string();

        self.pool.write(|conn| {
            conn.execute(
                "UPDATE windows SET focused = 0, updated_at = datetime('now') WHERE user_id = ?1 AND workspace_id = ?2 AND state != 'closed'",
                rusqlite::params![uid_str, wsid_str],
            ).map_err(|e| DbError::Query(e.to_string()))?;

            let max_z: i64 = conn
                .query_row(
                    "SELECT COALESCE(MAX(z_index), 0) FROM windows WHERE workspace_id = ?1 AND state != 'closed'",
                    [&wsid_str],
                    |row| row.get(0),
                ).unwrap_or(0);

            conn.execute(
                "UPDATE windows SET focused = 1, z_index = ?1, updated_at = datetime('now') WHERE id = ?2",
                rusqlite::params![max_z + 1, wid_str],
            ).map_err(|e| DbError::Query(e.to_string()))?;
            Ok(())
        }).map_err(WmError::Db)?;

        self.get_window(window_id).await
    }

    async fn get_window(&self, window_id: &str) -> Result<Window> {
        let wid = window_id.to_string();
        self.pool
            .read(|conn| {
                conn.query_row(
                    &format!(
                        "SELECT {} FROM windows WHERE id = ?1 AND state != 'closed'",
                        Self::WINDOW_COLS
                    ),
                    [&wid],
                    Self::row_to_window,
                )
                .map_err(|e| DbError::Query(e.to_string()))
            })
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("no rows") {
                    WmError::WindowNotFound(window_id.to_string())
                } else {
                    WmError::Db(e)
                }
            })
    }

    async fn list_windows(&self, user_id: &UserId, workspace_id: &str) -> Result<Vec<Window>> {
        let uid_str = user_id.0.to_string();
        let wsid_str = workspace_id.to_string();
        self.pool.read(|conn| {
            let mut stmt = conn.prepare(
                &format!("SELECT {} FROM windows WHERE user_id = ?1 AND workspace_id = ?2 AND state != 'closed' ORDER BY z_index DESC", Self::WINDOW_COLS),
            ).map_err(|e| DbError::Query(e.to_string()))?;

            let rows = stmt.query_map(rusqlite::params![uid_str, wsid_str], |row| {
                Self::row_to_window(row)
            }).map_err(|e| DbError::Query(e.to_string()))?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row.map_err(|e| DbError::Query(e.to_string()))?);
            }
            Ok(result)
        }).map_err(WmError::Db)
    }

    async fn create_workspace(
        &self,
        user_id: &UserId,
        req: CreateWorkspaceRequest,
    ) -> Result<Workspace> {
        let ws_id = WorkspaceId::new();
        let now = chrono::Utc::now().to_rfc3339();
        let uid_str = user_id.0.to_string();
        let wsid_str = ws_id.0.clone();
        let name = req.name.clone();

        let idx = self.pool.write(|conn| {
            let max_idx: i64 = conn
                .query_row(
                    "SELECT COALESCE(MAX(idx), -1) FROM workspaces WHERE user_id = ?1",
                    [&uid_str],
                    |row| row.get(0),
                ).unwrap_or(-1);

            conn.execute(
                "INSERT INTO workspaces (id, user_id, name, idx, active, created_at) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                rusqlite::params![wsid_str, uid_str, name, max_idx + 1, now],
            ).map_err(|e| DbError::Query(e.to_string()))?;

            Ok(max_idx + 1)
        }).map_err(WmError::Db)?;

        Ok(Workspace {
            id: ws_id,
            user_id: user_id.clone(),
            name: req.name,
            index: idx as u32,
            active: false,
            created_at: now,
        })
    }

    async fn switch_workspace(&self, user_id: &UserId, workspace_id: &str) -> Result<Workspace> {
        let uid_str = user_id.0.to_string();
        let wsid_str = workspace_id.to_string();

        let found = self
            .pool
            .write(|conn| {
                conn.execute(
                    "UPDATE workspaces SET active = 0 WHERE user_id = ?1",
                    [&uid_str],
                )
                .map_err(|e| DbError::Query(e.to_string()))?;

                let rows = conn
                    .execute(
                        "UPDATE workspaces SET active = 1 WHERE id = ?1 AND user_id = ?2",
                        rusqlite::params![wsid_str, uid_str],
                    )
                    .map_err(|e| DbError::Query(e.to_string()))?;

                Ok(rows > 0)
            })
            .map_err(WmError::Db)?;

        if !found {
            return Err(WmError::WorkspaceNotFound(workspace_id.to_string()));
        }

        self.pool
            .read(|conn| {
                conn.query_row(
                "SELECT id, user_id, name, idx, active, created_at FROM workspaces WHERE id = ?1",
                [&wsid_str],
                Self::row_to_workspace,
            ).map_err(|e| DbError::Query(e.to_string()))
            })
            .map_err(WmError::Db)
    }

    async fn list_workspaces(&self, user_id: &UserId) -> Result<Vec<Workspace>> {
        let uid_str = user_id.0.to_string();
        self.pool.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, user_id, name, idx, active, created_at FROM workspaces WHERE user_id = ?1 ORDER BY idx",
            ).map_err(|e| DbError::Query(e.to_string()))?;

            let rows = stmt.query_map([&uid_str], |row| {
                Self::row_to_workspace(row)
            }).map_err(|e| DbError::Query(e.to_string()))?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row.map_err(|e| DbError::Query(e.to_string()))?);
            }
            Ok(result)
        }).map_err(WmError::Db)
    }

    async fn get_active_workspace(&self, user_id: &UserId) -> Result<Workspace> {
        let uid_str = user_id.0.to_string();
        let result = self.pool.read(|conn| {
            conn.query_row(
                "SELECT id, user_id, name, idx, active, created_at FROM workspaces WHERE user_id = ?1 AND active = 1",
                [&uid_str],
                Self::row_to_workspace,
            ).map_err(|e| DbError::Query(e.to_string()))
        });

        match result {
            Ok(ws) => Ok(ws),
            Err(_) => {
                // Auto-create a default workspace
                self.create_workspace(
                    user_id,
                    CreateWorkspaceRequest {
                        name: "Workspace 1".to_string(),
                    },
                )
                .await?;
                let workspaces = self.list_workspaces(user_id).await?;
                if let Some(first) = workspaces.first() {
                    self.switch_workspace(user_id, &first.id.0).await
                } else {
                    Err(WmError::WorkspaceNotFound(
                        "failed to create default workspace".to_string(),
                    ))
                }
            }
        }
    }

    async fn delete_workspace(&self, workspace_id: &str) -> Result<()> {
        let wsid_str = workspace_id.to_string();
        let found = self
            .pool
            .write(|conn| {
                conn.execute("DELETE FROM windows WHERE workspace_id = ?1", [&wsid_str])
                    .map_err(|e| DbError::Query(e.to_string()))?;

                let rows = conn
                    .execute("DELETE FROM workspaces WHERE id = ?1", [&wsid_str])
                    .map_err(|e| DbError::Query(e.to_string()))?;

                Ok(rows > 0)
            })
            .map_err(WmError::Db)?;

        if !found {
            return Err(WmError::WorkspaceNotFound(workspace_id.to_string()));
        }
        Ok(())
    }
}

impl SqliteWindowManager {
    fn update_state(&self, window_id: &str, new_state: &str) -> Result<Window> {
        let wid = window_id.to_string();
        let rows = self.pool.write(|conn| {
            let rows = conn.execute(
                "UPDATE windows SET state = ?1, updated_at = datetime('now') WHERE id = ?2 AND state != 'closed'",
                rusqlite::params![new_state, wid],
            ).map_err(|e| DbError::Query(e.to_string()))?;
            Ok(rows)
        }).map_err(WmError::Db)?;

        if rows == 0 {
            // Check if window exists at all
            let exists: bool = self
                .pool
                .read(|conn| {
                    conn.query_row(
                        "SELECT COUNT(*) > 0 FROM windows WHERE id = ?1",
                        [&wid],
                        |row| row.get(0),
                    )
                    .map_err(|e| DbError::Query(e.to_string()))
                })
                .unwrap_or(false);

            if !exists {
                return Err(WmError::WindowNotFound(window_id.to_string()));
            }
            return Err(WmError::InvalidOperation("window is closed".to_string()));
        }

        self.pool
            .read(|conn| {
                conn.query_row(
                    &format!("SELECT {} FROM windows WHERE id = ?1", Self::WINDOW_COLS),
                    [&wid],
                    Self::row_to_window,
                )
                .map_err(|e| DbError::Query(e.to_string()))
            })
            .map_err(WmError::Db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_wm() -> SqliteWindowManager {
        let pool = cortex_db::Pool::open_in_memory().unwrap();
        let wm = SqliteWindowManager::new(pool);
        wm.init_schema().unwrap();
        wm
    }

    fn test_user() -> UserId {
        UserId(uuid::Uuid::new_v7(uuid::timestamp::Timestamp::now(
            uuid::NoContext,
        )))
    }

    /// Generate a valid UUID-based instance ID for tests.
    fn test_instance_id() -> String {
        uuid::Uuid::new_v7(uuid::timestamp::Timestamp::now(uuid::NoContext)).to_string()
    }

    #[tokio::test]
    async fn create_and_list_workspaces() {
        let wm = test_wm();
        let user = test_user();

        let ws1 = wm
            .create_workspace(
                &user,
                CreateWorkspaceRequest {
                    name: "Work".into(),
                },
            )
            .await
            .unwrap();
        let ws2 = wm
            .create_workspace(
                &user,
                CreateWorkspaceRequest {
                    name: "Personal".into(),
                },
            )
            .await
            .unwrap();

        let list = wm.list_workspaces(&user).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Work");
        assert_eq!(list[1].name, "Personal");
        assert_eq!(ws1.index, 0);
        assert_eq!(ws2.index, 1);
    }

    #[tokio::test]
    async fn switch_workspace() {
        let wm = test_wm();
        let user = test_user();

        let ws1 = wm
            .create_workspace(&user, CreateWorkspaceRequest { name: "W1".into() })
            .await
            .unwrap();
        let ws2 = wm
            .create_workspace(&user, CreateWorkspaceRequest { name: "W2".into() })
            .await
            .unwrap();

        wm.switch_workspace(&user, &ws1.id.0).await.unwrap();
        let active = wm.get_active_workspace(&user).await.unwrap();
        assert_eq!(active.name, "W1");

        wm.switch_workspace(&user, &ws2.id.0).await.unwrap();
        let active = wm.get_active_workspace(&user).await.unwrap();
        assert_eq!(active.name, "W2");
    }

    #[tokio::test]
    async fn auto_create_default_workspace() {
        let wm = test_wm();
        let user = test_user();

        let active = wm.get_active_workspace(&user).await.unwrap();
        assert_eq!(active.name, "Workspace 1");
        assert!(active.active);
    }

    #[tokio::test]
    async fn open_close_window() {
        let wm = test_wm();
        let user = test_user();

        let win = wm
            .open_window(
                &user,
                OpenWindowRequest {
                    instance_id: test_instance_id(),
                    title: "Test Window".into(),
                    x: 100,
                    y: 100,
                    width: 800,
                    height: 600,
                    workspace_id: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(win.title, "Test Window");
        assert_eq!(win.state, WindowState::Normal);
        assert!(win.focused);

        wm.close_window(&win.id.0).await.unwrap();
        let result = wm.get_window(&win.id.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn minimize_maximize_restore() {
        let wm = test_wm();
        let user = test_user();

        let win = wm
            .open_window(
                &user,
                OpenWindowRequest {
                    instance_id: test_instance_id(),
                    title: "State Test".into(),
                    x: 0,
                    y: 0,
                    width: 400,
                    height: 300,
                    workspace_id: None,
                },
            )
            .await
            .unwrap();

        let win = wm.minimize_window(&win.id.0).await.unwrap();
        assert_eq!(win.state, WindowState::Minimized);

        let win = wm.maximize_window(&win.id.0).await.unwrap();
        assert_eq!(win.state, WindowState::Maximized);

        let win = wm.restore_window(&win.id.0).await.unwrap();
        assert_eq!(win.state, WindowState::Normal);
    }

    #[tokio::test]
    async fn move_and_resize() {
        let wm = test_wm();
        let user = test_user();

        let win = wm
            .open_window(
                &user,
                OpenWindowRequest {
                    instance_id: test_instance_id(),
                    title: "Move Test".into(),
                    x: 0,
                    y: 0,
                    width: 400,
                    height: 300,
                    workspace_id: None,
                },
            )
            .await
            .unwrap();

        let win = wm
            .move_window(&win.id.0, MoveWindowRequest { x: 50, y: 75 })
            .await
            .unwrap();
        assert_eq!(win.x, 50);
        assert_eq!(win.y, 75);

        let win = wm
            .resize_window(
                &win.id.0,
                ResizeWindowRequest {
                    width: 1024,
                    height: 768,
                },
            )
            .await
            .unwrap();
        assert_eq!(win.width, 1024);
        assert_eq!(win.height, 768);
    }

    #[tokio::test]
    async fn focus_unfocuses_others() {
        let wm = test_wm();
        let user = test_user();

        let win1 = wm
            .open_window(
                &user,
                OpenWindowRequest {
                    instance_id: test_instance_id(),
                    title: "Win 1".into(),
                    x: 0,
                    y: 0,
                    width: 400,
                    height: 300,
                    workspace_id: None,
                },
            )
            .await
            .unwrap();

        let win2 = wm
            .open_window(
                &user,
                OpenWindowRequest {
                    instance_id: test_instance_id(),
                    title: "Win 2".into(),
                    x: 0,
                    y: 0,
                    width: 400,
                    height: 300,
                    workspace_id: None,
                },
            )
            .await
            .unwrap();

        let w1 = wm.get_window(&win1.id.0).await.unwrap();
        assert!(!w1.focused);
        let w2 = wm.get_window(&win2.id.0).await.unwrap();
        assert!(w2.focused);

        let w1 = wm.focus_window(&win1.id.0).await.unwrap();
        assert!(w1.focused);
        let w2 = wm.get_window(&win2.id.0).await.unwrap();
        assert!(!w2.focused);
    }

    #[tokio::test]
    async fn list_windows_by_workspace() {
        let wm = test_wm();
        let user = test_user();

        let ws1 = wm
            .create_workspace(&user, CreateWorkspaceRequest { name: "W1".into() })
            .await
            .unwrap();
        let ws2 = wm
            .create_workspace(&user, CreateWorkspaceRequest { name: "W2".into() })
            .await
            .unwrap();

        wm.open_window(
            &user,
            OpenWindowRequest {
                instance_id: test_instance_id(),
                title: "In WS1".into(),
                x: 0,
                y: 0,
                width: 400,
                height: 300,
                workspace_id: Some(ws1.id.0.clone()),
            },
        )
        .await
        .unwrap();

        wm.open_window(
            &user,
            OpenWindowRequest {
                instance_id: test_instance_id(),
                title: "In WS2".into(),
                x: 0,
                y: 0,
                width: 400,
                height: 300,
                workspace_id: Some(ws2.id.0.clone()),
            },
        )
        .await
        .unwrap();

        let ws1_wins = wm.list_windows(&user, &ws1.id.0).await.unwrap();
        let ws2_wins = wm.list_windows(&user, &ws2.id.0).await.unwrap();
        assert_eq!(ws1_wins.len(), 1);
        assert_eq!(ws2_wins.len(), 1);
        assert_eq!(ws1_wins[0].title, "In WS1");
        assert_eq!(ws2_wins[0].title, "In WS2");
    }

    #[tokio::test]
    async fn delete_workspace_closes_windows() {
        let wm = test_wm();
        let user = test_user();

        let ws = wm
            .create_workspace(
                &user,
                CreateWorkspaceRequest {
                    name: "To Delete".into(),
                },
            )
            .await
            .unwrap();
        wm.open_window(
            &user,
            OpenWindowRequest {
                instance_id: test_instance_id(),
                title: "Doomed".into(),
                x: 0,
                y: 0,
                width: 400,
                height: 300,
                workspace_id: Some(ws.id.0.clone()),
            },
        )
        .await
        .unwrap();

        wm.delete_workspace(&ws.id.0).await.unwrap();
        let list = wm.list_workspaces(&user).await.unwrap();
        assert!(list.iter().all(|w| w.id != ws.id));
    }

    #[tokio::test]
    async fn window_not_found_error() {
        let wm = test_wm();
        let result = wm.get_window("nonexistent").await;
        assert!(matches!(result, Err(WmError::WindowNotFound(_))));
    }

    #[tokio::test]
    async fn workspace_not_found_error() {
        let wm = test_wm();
        let user = test_user();
        let result = wm.switch_workspace(&user, "nonexistent").await;
        assert!(matches!(result, Err(WmError::WorkspaceNotFound(_))));
    }
}
