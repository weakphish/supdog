use anyhow::Result;
use chrono::NaiveDate;
use crate::db::Database;
use crate::models::TaskStatus;
use super::nodes::get_all_tasks;

/// Return count of open tasks from previous days (not today's note).
/// Carryover is a query-time operation — tasks from prior days automatically
/// appear in the task view since they are never deleted.
pub fn carry_over_tasks(db: &mut Database, today_note_id: &str, _today: NaiveDate) -> Result<usize> {
    let all_tasks = get_all_tasks(db, None)?;
    let open_from_other_days = all_tasks.iter().filter(|t| {
        let from_today = t.daily_note_id.as_deref() == Some(today_note_id);
        let is_open = matches!(t.status, Some(TaskStatus::Todo) | Some(TaskStatus::InProgress));
        !from_today && is_open
    }).count();
    Ok(open_from_other_days)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::queries::{daily_notes, nodes};
    use crate::models::{NodeType, TaskStatus, Priority};
    use chrono::NaiveDate;

    fn test_db() -> Database { Database::open_in_memory().unwrap() }

    #[test]
    fn test_carry_over_incomplete_tasks() {
        let mut db = test_db();
        let yesterday = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let today_date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();

        let yesterday_note = daily_notes::get_or_create(&mut db, yesterday).unwrap();
        let today_note = daily_notes::get_or_create(&mut db, today_date).unwrap();

        // Create a todo task from yesterday
        nodes::create(&mut db, nodes::CreateNode {
            parent_id: None,
            daily_note_id: Some(yesterday_note.id.clone()),
            content: "incomplete task".into(),
            node_type: NodeType::Task,
            position: 0,
            status: Some(TaskStatus::Todo),
            priority: Some(Priority::High),
            due_date: None,
        }).unwrap();

        let carried = carry_over_tasks(&mut db, &today_note.id, today_date).unwrap();
        assert_eq!(carried, 1);

        // calling again still returns 1 (query-time, not stored)
        let carried2 = carry_over_tasks(&mut db, &today_note.id, today_date).unwrap();
        assert_eq!(carried2, 1);
    }

    #[test]
    fn test_does_not_carry_done_tasks() {
        let mut db = test_db();
        let yesterday = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let today_date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();

        let yesterday_note = daily_notes::get_or_create(&mut db, yesterday).unwrap();
        let today_note = daily_notes::get_or_create(&mut db, today_date).unwrap();

        nodes::create(&mut db, nodes::CreateNode {
            parent_id: None,
            daily_note_id: Some(yesterday_note.id),
            content: "done task".into(),
            node_type: NodeType::Task,
            position: 0,
            status: Some(TaskStatus::Done),
            priority: None,
            due_date: None,
        }).unwrap();

        let carried = carry_over_tasks(&mut db, &today_note.id, today_date).unwrap();
        assert_eq!(carried, 0);
    }

    #[test]
    fn test_does_not_carry_todays_tasks() {
        let mut db = test_db();
        let today_date = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let today_note = daily_notes::get_or_create(&mut db, today_date).unwrap();

        nodes::create(&mut db, nodes::CreateNode {
            parent_id: None,
            daily_note_id: Some(today_note.id.clone()),
            content: "todays task".into(),
            node_type: NodeType::Task,
            position: 0,
            status: Some(TaskStatus::Todo),
            priority: None,
            due_date: None,
        }).unwrap();

        let carried = carry_over_tasks(&mut db, &today_note.id, today_date).unwrap();
        assert_eq!(carried, 0);
    }
}
