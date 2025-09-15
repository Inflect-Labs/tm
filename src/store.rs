use chrono::Utc;
use colored::Colorize;
use serde_json;
use std::fs;
use std::path::PathBuf;

use crate::models::{Project, ProjectStore, Task};
use crate::utils::get_data_file_path;

pub struct TaskStore {
    file_path: PathBuf,
    store: ProjectStore,
}

impl TaskStore {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = get_data_file_path()?;
        Ok(Self {
            file_path,
            store: ProjectStore {
                current_project: "default".to_string(),
                projects: vec![Project {
                    name: "default".to_string(),
                    tasks: Vec::new(),
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
                // Try to deserialize as old format (array of tasks) and migrate
                if let Ok(tasks) = serde_json::from_str::<Vec<Task>>(&content) {
                    self.store = ProjectStore {
                        current_project: "default".to_string(),
                        projects: vec![Project {
                            name: "default".to_string(),
                            tasks,
                            created_at: Utc::now(),
                        }],
                    };
                    // Save the migrated data
                    self.save()?;
                } else {
                    return Err("Invalid data format in tasks.json".into());
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

    pub fn get_current_tasks(&mut self) -> &mut Vec<Task> {
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
                    tasks: Vec::new(),
                    created_at: Utc::now(),
                });
            }
        }

        // Now safely get the current project's tasks
        self.store
            .projects
            .iter_mut()
            .find(|p| p.name == self.store.current_project)
            .map(|p| &mut p.tasks)
            .unwrap()
    }

    pub fn add_task(
        &mut self,
        path: Vec<usize>,
        text: String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let task = Task {
            text,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
            subtasks: Vec::new(),
        };

        let tasks = self.get_current_tasks();
        if path.is_empty() {
            tasks.push(task);
            self.save()?;
            Ok(true)
        } else {
            if let Some(parent) = self.find_item(path) {
                parent.subtasks.push(task);
                self.save()?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    pub fn find_item(&mut self, path: Vec<usize>) -> Option<&mut Task> {
        if path.is_empty() {
            return None;
        }

        let tasks = self.get_current_tasks();
        let mut parent_list = tasks;

        for &i in &path[..path.len() - 1] {
            if let Some(task) = parent_list.get_mut(i) {
                parent_list = &mut task.subtasks;
            } else {
                return None;
            }
        }

        parent_list.get_mut(path[path.len() - 1])
    }

    fn complete_dfs(task: &mut Task) {
        task.completed = true;
        task.completed_at = Some(Utc::now());

        for sub in task.subtasks.iter_mut() {
            Self::complete_dfs(sub);
        }
    }

    fn uncomplete_dfs(task: &mut Task) {
        task.completed = false;
        task.completed_at = None;

        for sub in task.subtasks.iter_mut() {
            Self::uncomplete_dfs(sub);
        }
    }

    pub fn complete_task(&mut self, path: Vec<usize>) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(task) = self.find_item(path) {
            Self::complete_dfs(task);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn uncomplete_task(&mut self, path: Vec<usize>) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(task) = self.find_item(path) {
            Self::uncomplete_dfs(task);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn delete_task(&mut self, path: Vec<usize>) -> Result<bool, Box<dyn std::error::Error>> {
        if path.is_empty() {
            return Ok(false);
        }

        let tasks = self.get_current_tasks();
        if path.len() == 1 {
            let index = path[0];
            if index < tasks.len() {
                tasks.remove(index);
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

    fn print_tasks(tasks: &Vec<Task>, depth: usize) {
        let indent = "  ".repeat(depth + 3);
        for (index, task) in tasks.iter().enumerate() {
            let status = if task.completed {
                "✓".green()
            } else {
                "○".red()
            };
            println!("{}[{}]  {}.  {}", indent, status, index, task.text);

            if !task.subtasks.is_empty() {
                Self::print_tasks(&task.subtasks, depth + 1);
            }
        }
    }

    pub fn list_tasks(&mut self) {
        let tasks = self.get_current_tasks();
        if tasks.is_empty() {
            println!("      list is empty.");
        } else {
            Self::print_tasks(tasks, 0);
        }
    }

    pub fn clear_completed(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tasks = self.get_current_tasks();
        Self::clear_completed_recursive(tasks);
        self.save()?;
        Ok(())
    }

    fn clear_completed_recursive(tasks: &mut Vec<Task>) {
        tasks.retain(|t| !t.completed);
        for task in tasks.iter_mut() {
            Self::clear_completed_recursive(&mut task.subtasks);
        }
    }

    pub fn clear_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tasks = self.get_current_tasks();
        tasks.clear();
        self.save()?;
        Ok(())
    }

    pub fn move_task(
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

        let tasks = self.get_current_tasks();
        let task_list = if parent_path.is_empty() {
            tasks
        } else {
            // Find the parent task item
            let mut parent_list = tasks;
            for &i in &parent_path {
                if let Some(task) = parent_list.get_mut(i) {
                    parent_list = &mut task.subtasks;
                } else {
                    return Ok(false);
                }
            }
            parent_list
        };

        if index >= task_list.len() {
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
                if index >= task_list.len() - 1 {
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
                if index >= task_list.len() - 1 {
                    return Ok(false); // Already at bottom
                }
                task_list.len() - 1
            }
            _ => {
                // Try to parse as a number for absolute positioning
                match direction.parse::<usize>() {
                    Ok(pos) => {
                        if pos >= task_list.len() {
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
            task_list.swap(index, new_index);
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
            tasks: Vec::new(),
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
