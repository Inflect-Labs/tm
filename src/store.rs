use chrono::Utc;
use colored::Colorize;
use serde_json;
use std::fs;
use std::path::PathBuf;

use crate::models::{Project, ProjectStore, Todo};
use crate::utils::get_data_file_path;

pub struct TodoStore {
    file_path: PathBuf,
    store: ProjectStore,
}

impl TodoStore {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
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

    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.store)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn get_current_todos(&mut self) -> &mut Vec<Todo> {
        // Ensure current project exists, create default if needed
        if !self
            .store
            .projects
            .iter()
            .any(|p| p.name == self.store.current_project)
        {
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
        self.store
            .projects
            .iter_mut()
            .find(|p| p.name == self.store.current_project)
            .map(|p| &mut p.todos)
            .unwrap()
    }

    pub fn add_todo(
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

    pub fn find_item(&mut self, path: Vec<usize>) -> Option<&mut Todo> {
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

    pub fn complete_todo(&mut self, path: Vec<usize>) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(todo) = self.find_item(path) {
            Self::complete_dfs(todo);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn delete_todo(&mut self, path: Vec<usize>) -> Result<bool, Box<dyn std::error::Error>> {
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

    pub fn list_todos(&mut self) {
        let todos = self.get_current_todos();
        if todos.is_empty() {
            println!("      list is empty.");
        } else {
            Self::print_todos(todos, 0);
        }
    }

    pub fn clear_completed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn clear_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let todos = self.get_current_todos();
        todos.clear();
        self.save()?;
        Ok(())
    }

    pub fn move_todo(
        &mut self,
        path: Vec<usize>,
        direction: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
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
    pub fn create_project(&mut self, name: String) -> Result<bool, Box<dyn std::error::Error>> {
        if self.store.projects.iter().any(|p| p.name == name) {
            return Ok(false); // Project already exists
        }

        self.store.projects.push(Project {
            name: name.clone(),
            todos: Vec::new(),
            created_at: Utc::now(),
        });
        Ok(true)
    }

    pub fn switch_project(&mut self, name: String) -> Result<bool, Box<dyn std::error::Error>> {
        if self.store.projects.iter().any(|p| p.name == name) {
            self.store.current_project = name;
            self.save()?;
            Ok(true)
        } else {
            Ok(false) // Project doesn't exist
        }
    }

    pub fn list_projects(&self) {
        for project in &self.store.projects {
            let marker = if project.name == self.store.current_project {
                " * ".green()
            } else {
                "   ".normal()
            };
            println!("{}{}", marker, project.name);
        }
    }

    pub fn delete_project(&mut self, name: String) -> Result<bool, Box<dyn std::error::Error>> {
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

    pub fn get_current_project_name(&self) -> &str {
        &self.store.current_project
    }
}
