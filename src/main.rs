use clap::Parser;

mod commands;
mod handlers;
mod models;
mod store;
mod utils;

use commands::Commands;
use handlers::*;
use store::TodoStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for version flags first
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && (args[1] == "-v" || args[1] == "--version") {
        handle_version();
        return Ok(());
    }

    let commands = Commands::parse();

    let mut store = TodoStore::new()?;
    store.load()?;

    match commands {
        Commands::Add { path, text } => {
            handle_add(&mut store, path, text)?;
        }
        Commands::List => {
            handle_list(&mut store);
        }
        Commands::Clear => {
            handle_clear(&mut store)?;
        }
        Commands::Delete { path } => {
            handle_delete(&mut store, path)?;
        }
        Commands::Check { path } => {
            handle_check(&mut store, path)?;
        }
        Commands::ClearAll => {
            handle_clear_all(&mut store)?;
        }
        Commands::Move {
            path,
            up,
            down,
            top,
            bottom,
            position,
        } => {
            handle_move(&mut store, path, up, down, top, bottom, position)?;
        }
        Commands::CreateProject { name } => {
            handle_create_project(&mut store, name)?;
        }
        Commands::SwitchProject { name } => {
            handle_switch_project(&mut store, name)?;
        }
        Commands::ListProjects => {
            handle_list_projects(&store);
        }
        Commands::DeleteProject { name } => {
            handle_delete_project(&mut store, name)?;
        }
        Commands::Update => {
            handle_update()?;
        }
        Commands::Uninstall { yes } => {
            handle_uninstall(yes)?;
        }
        Commands::Version => {
            handle_version();
        }
    }

    Ok(())
}
