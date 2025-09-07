use chrono::{DateTime, Utc};
use clap::Parser;
use colored::Colorize;
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

fn get_data_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_dir().ok_or("Could not determine data directory")?;

    let app_dir = data_dir.join("td");

    // Create the directory if it doesn't exist
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }

    Ok(app_dir.join("todos.json"))
}

struct TodoStore {
    file_path: PathBuf,
    todos: Vec<Todo>,
}

impl TodoStore {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = get_data_file_path()?;
        Ok(Self {
            file_path,
            todos: Vec::new(),
        })
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
    let commands = Commands::parse();

    let mut store = TodoStore::new()?;
    store.load()?;

    match commands {
        Commands::Add { text } => {
            store.add_todo(text)?;
        }
        Commands::List => {
            let todos = store.list_todos();
            if todos.is_empty() {
                println!("list is empty.");
            } else {
                for todo in todos {
                    let status = if todo.completed {
                        "✓".green()
                    } else {
                        "○".red()
                    };
                    println!("[{}], {}, {}", status, todo.id, todo.text);
                }
            }
        }
        Commands::Clear => {
            let _ = store.clear_completed()?;
        }
        Commands::Delete { id } => {
            let _ = store.delete_todo(id)?;
        }
        Commands::Complete { id } => {
            let _ = store.complete_todo(id)?;
        }
    }
}
