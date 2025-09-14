use clap::Parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "tm")]
#[command(about = "A simple and powerful task manager CLI")]
#[command(version = VERSION)]
#[command(arg_required_else_help = true)]
pub enum Commands {
    /// add a new task or subtask
    #[command(visible_alias = "a")]
    Add {
        /// description of the task
        text: String,
        /// nested index path of the parent task (empty for root level)
        #[arg(required = false)]
        path: Vec<usize>,
    },
    /// list all tasks
    #[command(visible_alias = "l", visible_alias = "ls")]
    List,
    /// mark an item as completed
    #[command(visible_alias = "c")]
    Check {
        /// the nested index path of the task to complete
        #[arg(required = true, num_args = 1..)]
        path: Vec<usize>,
    },
    /// delete a task
    #[command(visible_alias = "d", visible_alias = "rm")]
    Delete {
        /// the nested index path of the task to delete
        #[arg(required = true, num_args = 1..)]
        path: Vec<usize>,
    },
    /// clear all completed tasks
    #[command(visible_alias = "cl")]
    Clear,
    /// clear all tasks
    #[command(visible_alias = "ca")]
    ClearAll,
    /// move a task up or down in the list
    #[command(visible_alias = "m")]
    Move {
        /// the nested index path of the task to move
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
    /// update TM CLI to the latest version
    Update,
    /// print version information
    Version,
    /// completely remove TM CLI and all its data
    Uninstall {
        /// skip confirmation prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
}
