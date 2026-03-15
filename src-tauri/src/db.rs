use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

pub fn init_db(app_data_dir: PathBuf) -> Result<Connection, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(&app_data_dir)?;
    let db_path = app_data_dir.join("supdog.db");
    let mut conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    let migrations = Migrations::new(vec![
        M::up(include_str!("migrations/001_initial.sql")),
    ]);
    migrations.to_latest(&mut conn)?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_migration_runs() {
        let tmp = TempDir::new().unwrap();
        let conn = init_db(tmp.path().to_path_buf()).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='blocks'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_all_tables_created() {
        let tmp = TempDir::new().unwrap();
        let conn = init_db(tmp.path().to_path_buf()).unwrap();
        let tables = ["daily_notes", "blocks", "tags", "block_tags", "block_links", "mind_maps", "mind_map_nodes"];
        for table in tables {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1", [table], |r| r.get(0))
                .unwrap();
            assert_eq!(count, 1, "Table {} not found", table);
        }
    }
}
