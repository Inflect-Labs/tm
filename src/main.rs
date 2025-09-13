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

#[derive(Serialize, Deserialize, Clone)]
struct Project {
    name: String,
    todos: Vec<Todo>,
    created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct ProjectStore {
    current_project: String,
    projects: Vec<Project>,
}

fn get_data_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_dir().ok_or("could not determine data directory")?;

    let app_dir = data_dir.join("td");

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }

    Ok(app_dir.join("todos.json"))
}

fn get_data_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_dir().ok_or("could not determine data directory")?;
    Ok(data_dir.join("td"))
}

struct TodoStore {
    file_path: PathBuf,
    store: ProjectStore,
}

impl TodoStore {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = get_data_file_path()?;
        Ok(Self {
            file_path,
            store: ProjectStore {
                current_project: "default".to_string(),
                projects: vec![Project {
                    name: "default".to_string(),
                    todos: Vec::new(),
                    created_at: Utc::now(),
                }],
            },
        })
    }

    fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.file_path.exists() {
            let content = fs::read_to_string(&self.file_path)?;
            
            // Try to deserialize as new format first
            if let Ok(store) = serde_json::from_str::<ProjectStore>(&content) {
                self.store = store;
            } else {
                // Try to deserialize as old format (array of todos) and migrate
                if let Ok(todos) = serde_json::from_str::<Vec<Todo>>(&content) {
                    self.store = ProjectStore {
                        current_project: "default".to_string(),
                        projects: vec![Project {
                            name: "default".to_string(),
                            todos,
                            created_at: Utc::now(),
                        }],
                    };
                    // Save the migrated data
                    self.save()?;
                } else {
                    return Err("Invalid data format in todos.json".into());
                }
            }
        }
        Ok(())
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.store)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    fn get_current_todos(&mut self) -> &mut Vec<Todo> {
        // Ensure current project exists, create default if needed
        if !self.store.projects.iter().any(|p| p.name == self.store.current_project) {
            self.store.current_project = "default".to_string();
            if !self.store.projects.iter().any(|p| p.name == "default") {
                self.store.projects.push(Project {
                    name: "default".to_string(),
                    todos: Vec::new(),
                    created_at: Utc::now(),
                });
            }
        }
        
        // Now safely get the current project's todos
        self.store.projects.iter_mut()
            .find(|p| p.name == self.store.current_project)
            .map(|p| &mut p.todos)
            .unwrap()
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

        let todos = self.get_current_todos();
        if path.is_empty() {
            todos.push(todo);
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

        let todos = self.get_current_todos();
        let mut parent_list = todos;

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

        let todos = self.get_current_todos();
        if path.len() == 1 {
            let index = path[0];
            if index < todos.len() {
                todos.remove(index);
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

    fn list_todos(&mut self) {
        let todos = self.get_current_todos();
        if todos.is_empty() {
            println!("      list is empty.");
        } else {
            Self::print_todos(todos, 0);
        }
    }

    fn clear_completed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let todos = self.get_current_todos();
        Self::clear_completed_recursive(todos);
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
        let todos = self.get_current_todos();
        todos.clear();
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

        let todos = self.get_current_todos();
        let todo_list = if parent_path.is_empty() {
            todos
        } else {
            // Find the parent todo item
            let mut parent_list = todos;
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

    // Project management methods
    fn create_project(&mut self, name: String) -> Result<bool, Box<dyn std::error::Error>> {
        if self.store.projects.iter().any(|p| p.name == name) {
            return Ok(false); // Project already exists
        }
        
        self.store.projects.push(Project {
            name: name.clone(),
            todos: Vec::new(),
            created_at: Utc::now(),
        });
        self.save()?;
        Ok(true)
    }

    fn switch_project(&mut self, name: String) -> Result<bool, Box<dyn std::error::Error>> {
        if self.store.projects.iter().any(|p| p.name == name) {
            self.store.current_project = name;
            self.save()?;
            Ok(true)
        } else {
            Ok(false) // Project doesn't exist
        }
    }

    fn list_projects(&self) {
        for project in &self.store.projects {
            let marker = if project.name == self.store.current_project {
                " * ".green()
            } else {
                "   ".normal()
            };
            println!("{}{}", marker, project.name);
        }
    }

    fn delete_project(&mut self, name: String) -> Result<bool, Box<dyn std::error::Error>> {
        if name == "default" {
            return Ok(false); // Cannot delete default project
        }
        
        if let Some(pos) = self.store.projects.iter().position(|p| p.name == name) {
            self.store.projects.remove(pos);
            
            // If we deleted the current project, switch to default
            if self.store.current_project == name {
                self.store.current_project = "default".to_string();
            }
            
            self.save()?;
            Ok(true)
        } else {
            Ok(false) // Project doesn't exist
        }
    }

    fn get_current_project_name(&self) -> &str {
        &self.store.current_project
    }
}

#[derive(Parser)]
enum Commands {
    /// add a new todo item or subtask
    #[command(visible_alias = "a")]
    Add {
        /// description of the item
        text: String,
        /// nested index path of the parent item (empty for root level)
        #[arg(required = false)]
        path: Vec<usize>,
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
        /// move up one position
        #[arg(short = 'u', long = "up")]
        up: bool,
        /// move down one position
        #[arg(short = 'd', long = "down")]
        down: bool,
        /// move to top
        #[arg(short = 't', long = "top")]
        top: bool,
        /// move to bottom
        #[arg(short = 'b', long = "bottom")]
        bottom: bool,
        /// specific position to move to
        #[arg(short = 'p', long = "position")]
        position: Option<usize>,
    },
    /// create a new project
    #[command(visible_alias = "cp")]
    CreateProject {
        /// name of the project to create
        name: String,
    },
    /// switch to a different project
    #[command(visible_alias = "sp")]
    SwitchProject {
        /// name of the project to switch to
        name: String,
    },
    /// list all available projects
    #[command(visible_alias = "lp")]
    ListProjects,
    /// delete a project
    #[command(visible_alias = "dp")]
    DeleteProject {
        /// name of the project to delete
        name: String,
    },
    /// completely remove TD CLI and all its data
    Uninstall {
        /// skip confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,
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
            println!("      Current: {}", store.get_current_project_name().green());
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
        Commands::Move { path, up, down, top, bottom, position } => {
            // Determine the direction based on the flags
            let direction = if up {
                "up".to_string()
            } else if down {
                "down".to_string()
            } else if top {
                "top".to_string()
            } else if bottom {
                "bottom".to_string()
            } else if let Some(pos) = position {
                pos.to_string()
            } else {
                eprintln!("error: must specify a direction flag (-u, -d, -t, -b) or position (-p)");
                std::process::exit(1);
            };

            if store.move_todo(path.clone(), &direction)? {
                println!("moved item {} {}", format_path(&path), direction);
            } else {
                eprintln!("error: could not move item at path {}", format_path(&path));
                std::process::exit(1);
            }
        }
        Commands::CreateProject { name } => {
            if store.create_project(name.clone())? {
                println!("created project '{}'", name);
            } else {
                eprintln!("error: project '{}' already exists", name);
                std::process::exit(1);
            }
        }
        Commands::SwitchProject { name } => {
            if store.switch_project(name.clone())? {
                println!("switched to project '{}'", name);
            } else {
                eprintln!("error: project '{}' not found", name);
                std::process::exit(1);
            }
        }
        Commands::ListProjects => {
            store.list_projects();
        }
        Commands::DeleteProject { name } => {
            if store.delete_project(name.clone())? {
                println!("deleted project '{}'", name);
            } else {
                eprintln!("error: project '{}' not found or cannot be deleted", name);
                std::process::exit(1);
            }
        }
        Commands::Uninstall { yes } => {
            let data_dir = get_data_directory()?;
            
            if !yes {
                println!("⚠️  This will permanently delete ALL your todo data!");
                println!("   Data location: {}", data_dir.display());
                println!("");
                print!("Are you sure you want to continue? (y/N): ");
                use std::io::{self, Write};
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Uninstall cancelled.");
                    return Ok(());
                }
            }
            
            if data_dir.exists() {
                fs::remove_dir_all(&data_dir)?;
                println!("✓ Removed all todo data from {}", data_dir.display());
            } else {
                println!("No data found to remove");
            }
            
            println!("");
            println!("To remove the TD CLI binary, run:");
            println!("  cargo uninstall td");
            println!("");
            println!("TD CLI has been uninstalled successfully!");
        }
    }

    Ok(())
}
