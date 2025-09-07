use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

struct TodoStore {
    file_path: PathBuf,
    todos: Vec<Todo>,
}

impl TodoStore {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            todos: Vec::new(),
        }
    }

    fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.file_path.exists() {
            let content = fs::read_to_string(&self.file_path)?;
            self.todos = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.todos)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    fn add_todo(&mut self, text: String) -> Result<(), Box<dyn std::error::Error>> {
        let id = self.todos.len() + 1;
        let todo = Todo {
            id,
            text,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
        };
        self.todos.push(todo);
        self.save()?;
        Ok(())
    }

    fn complete_todo(&mut self, id: usize) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = true;
            todo.completed_at = Some(Utc::now());
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn delete_todo(&mut self, id: usize) -> Result<bool, Box<dyn std::error::Error>> {
        let initial_len = self.todos.len();
        self.todos.retain(|t| t.id != id);
        if self.todos.len() < initial_len {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn clear_completed(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let initial_len = self.todos.len();
        self.todos.retain(|t| !t.completed);
        let removed_count = initial_len - self.todos.len();
        if removed_count > 0 {
            self.save()?;
        }
        Ok(removed_count)
    }

    fn list_todos(&self) -> &Vec<Todo> {
        &self.todos
    }
}

#[derive(Parser)]
enum Commands {
    Add { text: String },
    List,
    Complete { id: usize },
    Delete { id: usize },
    Clear,
}

fn main() {
    println!("Hello, world!");
}
