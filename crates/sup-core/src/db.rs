use anyhow::Result;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let mut db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let mut db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&mut self) -> Result<()> {
        let migrations = Migrations::new(vec![
            M::up(include_str!("migrations/001_initial.sql")),
        ]);
        migrations.to_latest(&mut self.conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_db_creates_schema() {
        let db = Database::open_in_memory().unwrap();
        let count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='nodes'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_all_tables_exist() {
        let db = Database::open_in_memory().unwrap();
        for table in &["daily_notes", "nodes", "tags", "node_tags", "node_links"] {
            let count: i64 = db.conn.query_row(
                &format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}'", table),
                [],
                |r| r.get(0),
            ).unwrap();
            assert_eq!(count, 1, "table {} missing", table);
        }
    }
}
