use anyhow::{bail, Result};
use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyNote {
    pub id: String,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    Bullet,
    H1,
    H2,
    H3,
    Quote,
    Code,
    Task,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Bullet => "bullet",
            NodeType::H1 => "h1",
            NodeType::H2 => "h2",
            NodeType::H3 => "h3",
            NodeType::Quote => "quote",
            NodeType::Code => "code",
            NodeType::Task => "task",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "bullet" => Ok(NodeType::Bullet),
            "h1" => Ok(NodeType::H1),
            "h2" => Ok(NodeType::H2),
            "h3" => Ok(NodeType::H3),
            "quote" => Ok(NodeType::Quote),
            "code" => Ok(NodeType::Code),
            "task" => Ok(NodeType::Task),
            _ => bail!("unknown node type: {}", s),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NodeType::Bullet => "•",
            NodeType::H1 | NodeType::H2 | NodeType::H3 => "#",
            NodeType::Quote => "\"",
            NodeType::Code => ">",
            NodeType::Task => "☐",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "todo" => Ok(TaskStatus::Todo),
            "in_progress" => Ok(TaskStatus::InProgress),
            "done" => Ok(TaskStatus::Done),
            "cancelled" => Ok(TaskStatus::Cancelled),
            _ => bail!("unknown status: {}", s),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            TaskStatus::Todo => TaskStatus::InProgress,
            TaskStatus::InProgress => TaskStatus::Done,
            TaskStatus::Done => TaskStatus::Cancelled,
            TaskStatus::Cancelled => TaskStatus::Todo,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "☐",
            TaskStatus::InProgress => "◐",
            TaskStatus::Done => "☑",
            TaskStatus::Cancelled => "✗",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    High,
    Med,
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::High => "high",
            Priority::Med => "med",
            Priority::Low => "low",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "high" => Ok(Priority::High),
            "med" => Ok(Priority::Med),
            "low" => Ok(Priority::Low),
            _ => bail!("unknown priority: {}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub parent_id: Option<String>,
    pub daily_note_id: Option<String>,
    pub content: String,
    pub node_type: NodeType,
    pub position: i64,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_roundtrip() {
        assert_eq!(NodeType::Task.as_str(), "task");
        assert_eq!(NodeType::from_str("task").unwrap(), NodeType::Task);
        assert!(NodeType::from_str("bogus").is_err());
    }

    #[test]
    fn test_task_status_cycle() {
        assert_eq!(TaskStatus::Todo.next(), TaskStatus::InProgress);
        assert_eq!(TaskStatus::InProgress.next(), TaskStatus::Done);
        assert_eq!(TaskStatus::Done.next(), TaskStatus::Cancelled);
        assert_eq!(TaskStatus::Cancelled.next(), TaskStatus::Todo);
    }

    #[test]
    fn test_all_node_types_have_icons() {
        for t in &[NodeType::Bullet, NodeType::H1, NodeType::H2, NodeType::H3,
                   NodeType::Quote, NodeType::Code, NodeType::Task] {
            assert!(!t.icon().is_empty());
        }
    }

    #[test]
    fn test_all_statuses_have_icons() {
        for s in &[TaskStatus::Todo, TaskStatus::InProgress,
                   TaskStatus::Done, TaskStatus::Cancelled] {
            assert!(!s.icon().is_empty());
        }
    }
}
