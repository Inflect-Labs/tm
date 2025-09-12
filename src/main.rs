use chrono::{DateTime, Utc};
use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
struct Todo {
    text: String,
    completed: bool,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    subtasks: Vec<Todo>,
}

fn get_data_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_dir().ok_or("could not determine data directory")?;

    let app_dir = data_dir.join("td");

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

    fn add_todo(
        &mut self,
        path: Vec<usize>,
        text: String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let todo = Todo {
            text,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
            subtasks: Vec::new(),
        };

        if path.is_empty() {
            self.todos.push(todo);
            self.save()?;
            Ok(true)
        } else {
            if let Some(parent) = self.find_item(path) {
                parent.subtasks.push(todo);
                self.save()?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    fn find_item(&mut self, path: Vec<usize>) -> Option<&mut Todo> {
        if path.is_empty() {
            return None;
        }

        let mut parent_list = &mut self.todos;

        for &i in &path[..path.len() - 1] {
            if let Some(todo) = parent_list.get_mut(i) {
                parent_list = &mut todo.subtasks;
            } else {
                return None;
            }
        }

        parent_list.get_mut(path[path.len() - 1])
    }

    fn complete_dfs(todo: &mut Todo) {
        todo.completed = true;
        todo.completed_at = Some(Utc::now());

        for sub in todo.subtasks.iter_mut() {
            Self::complete_dfs(sub);
        }
    }

    fn complete_todo(&mut self, path: Vec<usize>) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(todo) = self.find_item(path) {
            Self::complete_dfs(todo);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn delete_todo(&mut self, path: Vec<usize>) -> Result<bool, Box<dyn std::error::Error>> {
        if path.is_empty() {
            return Ok(false);
        }

        if path.len() == 1 {
            let index = path[0];
            if index < self.todos.len() {
                self.todos.remove(index);
                self.save()?;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            let parent_path = path[..path.len() - 1].to_vec();
            let index = path[path.len() - 1];

            if let Some(parent) = self.find_item(parent_path) {
                if index < parent.subtasks.len() {
                    parent.subtasks.remove(index);
                    self.save()?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
    }

    fn print_todos(todos: &Vec<Todo>, depth: usize) {
        let indent = "  ".repeat(depth + 3);
        for (index, todo) in todos.iter().enumerate() {
            let status = if todo.completed {
                "✓".green()
            } else {
                "○".red()
            };
            println!("{}[{}]  {}.  {}", indent, status, index, todo.text);

            if !todo.subtasks.is_empty() {
                Self::print_todos(&todo.subtasks, depth + 1);
            }
        }
    }

    fn list_todos(&self) {
        if self.todos.is_empty() {
            println!("      list is empty.");
        } else {
            Self::print_todos(&self.todos, 0);
        }
    }

    fn clear_completed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Self::clear_completed_recursive(&mut self.todos);
        self.save()?;
        Ok(())
    }

    fn clear_completed_recursive(todos: &mut Vec<Todo>) {
        todos.retain(|t| !t.completed);
        for todo in todos.iter_mut() {
            Self::clear_completed_recursive(&mut todo.subtasks);
        }
    }

    fn clear_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.todos.clear();
        self.save()?;
        Ok(())
    }

    fn move_todo(&mut self, path: Vec<usize>, direction: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if path.is_empty() {
            return Ok(false);
        }

        let index = path[path.len() - 1];
        let parent_path = if path.len() == 1 {
            Vec::new()
        } else {
            path[..path.len() - 1].to_vec()
        };

        let todo_list = if parent_path.is_empty() {
            &mut self.todos
        } else {
            // Find the parent todo item
            let mut parent_list = &mut self.todos;
            for &i in &parent_path {
                if let Some(todo) = parent_list.get_mut(i) {
                    parent_list = &mut todo.subtasks;
                } else {
                    return Ok(false);
                }
            }
            parent_list
        };

        if index >= todo_list.len() {
            return Ok(false);
        }

        let new_index = match direction.to_lowercase().as_str() {
            "up" => {
                if index == 0 {
                    return Ok(false); // Already at top
                }
                index - 1
            }
            "down" => {
                if index >= todo_list.len() - 1 {
                    return Ok(false); // Already at bottom
                }
                index + 1
            }
            "top" => {
                if index == 0 {
                    return Ok(false); // Already at top
                }
                0
            }
            "bottom" => {
                if index >= todo_list.len() - 1 {
                    return Ok(false); // Already at bottom
                }
                todo_list.len() - 1
            }
            _ => {
                // Try to parse as a number for absolute positioning
                match direction.parse::<usize>() {
                    Ok(pos) => {
                        if pos >= todo_list.len() {
                            return Ok(false);
                        }
                        pos
                    }
                    Err(_) => return Ok(false),
                }
            }
        };

        // Perform the swap
        if new_index != index {
            todo_list.swap(index, new_index);
            self.save()?;
        }

        Ok(true)
    }
}

#[derive(Parser)]
enum Commands {
    /// add a new todo item or subtask
    #[command(visible_alias = "a")]
    Add {
        /// nested index path of the parent item (empty for root level)
        path: Vec<usize>,
        /// description of the item
        text: String,
    },
    /// list all todo items
    #[command(visible_alias = "l", visible_alias = "ls")]
    List,
    /// mark an item as completed
    #[command(visible_alias = "c")]
    Check {
        /// the nested index path of the item to complete
        #[arg(required = true, num_args = 1..)]
        path: Vec<usize>,
    },
    /// delete an item
    #[command(visible_alias = "d", visible_alias = "rm")]
    Delete {
        /// the nested index path of the item to delete
        #[arg(required = true, num_args = 1..)]
        path: Vec<usize>,
    },
    /// clear all completed items
    #[command(visible_alias = "cl")]
    Clear,
    /// clear all items
    #[command(visible_alias = "ca")]
    ClearAll,
    /// move an item up or down in the list
    #[command(visible_alias = "m")]
    Move {
        /// the nested index path of the item to move
        #[arg(required = true, num_args = 1..)]
        path: Vec<usize>,
        /// direction to move: up, down, top, bottom, or specific position
        direction: String,
    },
}

fn format_path(path: &Vec<usize>) -> String {
    path.iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let commands = Commands::parse();

    let mut store = TodoStore::new()?;
    store.load()?;

    match commands {
        Commands::Add { path, text } => {
            if store.add_todo(path.clone(), text)? {
                if path.is_empty() {
                    println!("added todo item");
                } else {
                    println!("added subtask to item {}", format_path(&path));
                }
            } else {
                eprintln!(
                    "error: parent item at path {} not found",
                    format_path(&path)
                );
                std::process::exit(1);
            }
        }
        Commands::List => {
            println!("");
            println!("");
            store.list_todos();
            println!("");
            println!("");
        }
        Commands::Clear => {
            store.clear_completed()?;
            println!("cleared completed items");
        }
        Commands::Delete { path } => {
            if store.delete_todo(path.clone())? {
                println!("deleted item {}", format_path(&path));
            } else {
                eprintln!("error: item at path {} not found", format_path(&path));
                std::process::exit(1);
            }
        }
        Commands::Check { path } => {
            if store.complete_todo(path.clone())? {
                println!("completed item {}", format_path(&path));
            } else {
                eprintln!("error: item at path {} not found", format_path(&path));
                std::process::exit(1);
            }
        }
        Commands::ClearAll => {
            store.clear_all()?;
            println!("cleared all items");
        }
        Commands::Move { path, direction } => {
            if store.move_todo(path.clone(), &direction)? {
                println!("moved item {} {}", format_path(&path), direction);
            } else {
                eprintln!("error: could not move item at path {}", format_path(&path));
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
