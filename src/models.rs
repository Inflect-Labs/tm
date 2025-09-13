use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Todo {
    pub text: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub subtasks: Vec<Todo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub todos: Vec<Todo>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectStore {
    pub current_project: String,
    pub projects: Vec<Project>,
}
