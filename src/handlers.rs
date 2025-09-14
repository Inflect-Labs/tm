use colored::Colorize;
use std::fs;
use std::process::Command;

use crate::store::TaskStore;
use crate::utils::{format_path, get_data_directory};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTALL_SCRIPT_URL: &str = "https://tm-cli.com/install";

pub fn handle_add(
    store: &mut TaskStore,
    path: Vec<usize>,
    text: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.add_task(path.clone(), text)? {
        if path.is_empty() {
            println!("added task item");
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
    Ok(())
}

pub fn handle_list(store: &mut TaskStore) {
    println!("");
    println!(
        "      Current: {}",
        store.get_current_project_name().green()
    );
    println!("");
    store.list_tasks();
    println!("");
    println!("");
}

pub fn handle_check(
    store: &mut TaskStore,
    path: Vec<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.complete_task(path.clone())? {
        println!("completed item {}", format_path(&path));
    } else {
        eprintln!("error: item at path {} not found", format_path(&path));
        std::process::exit(1);
    }
    Ok(())
}

pub fn handle_delete(
    store: &mut TaskStore,
    path: Vec<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.delete_task(path.clone())? {
        println!("deleted item {}", format_path(&path));
    } else {
        eprintln!("error: item at path {} not found", format_path(&path));
        std::process::exit(1);
    }
    Ok(())
}

pub fn handle_clear(store: &mut TaskStore) -> Result<(), Box<dyn std::error::Error>> {
    store.clear_completed()?;
    println!("cleared completed items");
    Ok(())
}

pub fn handle_clear_all(store: &mut TaskStore) -> Result<(), Box<dyn std::error::Error>> {
    store.clear_all()?;
    println!("cleared all items");
    Ok(())
}

pub fn handle_move(
    store: &mut TaskStore,
    path: Vec<usize>,
    up: bool,
    down: bool,
    top: bool,
    bottom: bool,
    position: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    if store.move_task(path.clone(), &direction)? {
        println!("moved item {} {}", format_path(&path), direction);
    } else {
        eprintln!("error: could not move item at path {}", format_path(&path));
        std::process::exit(1);
    }
    Ok(())
}

pub fn handle_create_project(
    store: &mut TaskStore,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.create_project(name.clone())? {
        println!("created project '{}'", name);
    } else {
        eprintln!("error: project '{}' already exists", name);
        std::process::exit(1);
    }
    Ok(())
}

pub fn handle_switch_project(
    store: &mut TaskStore,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.switch_project(name.clone())? {
        println!("switched to project '{}'", name);
    } else {
        eprintln!("error: project '{}' not found", name);
        std::process::exit(1);
    }
    Ok(())
}

pub fn handle_list_projects(store: &TaskStore) {
    store.list_projects();
}

pub fn handle_delete_project(
    store: &mut TaskStore,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.delete_project(name.clone())? {
        println!("deleted project '{}'", name);
    } else {
        eprintln!("error: project '{}' not found or cannot be deleted", name);
        std::process::exit(1);
    }
    Ok(())
}

pub fn handle_update() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Checking for updates...");
    println!("Current version: {}", VERSION.green());

    // Check latest version from our API
    let latest_version = match get_latest_version() {
        Ok(version) => version,
        Err(e) => {
            println!("⚠️  Could not check latest version: {}", e);
            println!("Proceeding with update anyway...");
            "unknown".to_string()
        }
    };

    if latest_version != "unknown" {
        println!("Latest version: {}", latest_version.green());
        
        // Compare versions (remove 'v' prefix if present)
        let current_clean = VERSION.trim_start_matches('v');
        let latest_clean = latest_version.trim_start_matches('v');
        
        if current_clean == latest_clean {
            println!("✅ You're already running the latest version!");
            return Ok(());
        }
    }

    println!("");
    println!("Downloading and running the latest installer...");

    let output = Command::new("bash")
        .arg("-c")
        .arg(&format!("curl -fsSL {} | bash", INSTALL_SCRIPT_URL))
        .output()?;

    if output.status.success() {
        println!("✅ Update completed successfully!");
        println!("Run 'tm --version' to verify the new version.");
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ Update failed: {}", error_msg);
        eprintln!("");
        eprintln!("You can try updating manually:");
        eprintln!("  curl -fsSL {} | bash", INSTALL_SCRIPT_URL);
        std::process::exit(1);
    }

    Ok(())
}

fn get_latest_version() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("curl")
        .arg("-sL")
        .arg("https://tm-cli.com/api/version")
        .output()?;

    if !output.status.success() {
        return Err("Failed to fetch version from API".into());
    }

    let response = String::from_utf8(output.stdout)?;
    
    // Simple JSON parsing to extract version
    if let Some(start) = response.find("\"version\":\"") {
        let start = start + 11; // Length of "\"version\":\""
        if let Some(end) = response[start..].find("\"") {
            let version = &response[start..start + end];
            return Ok(version.to_string());
        }
    }
    
    Err("Could not parse version from API response".into())
}

pub fn handle_version() {
    println!("tm {}", VERSION);
}

pub fn handle_uninstall(yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = get_data_directory()?;

    // Get the current binary path
    let current_exe = std::env::current_exe()?;

    if !yes {
        println!("⚠️  This will permanently delete:");
        println!("   • ALL your task data: {}", data_dir.display());
        println!("   • TM CLI binary: {}", current_exe.display());
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

    // Remove data directory
    if data_dir.exists() {
        fs::remove_dir_all(&data_dir)?;
        println!("✓ Removed all task data from {}", data_dir.display());
    } else {
        println!("No data found to remove");
    }

    // Remove the binary itself
    println!("✓ Removing TM CLI binary from {}", current_exe.display());

    // We need to delete ourselves, which requires special handling
    #[cfg(unix)]
    {
        // On Unix systems, we can delete the file while it's running
        if let Err(e) = fs::remove_file(&current_exe) {
            println!("⚠️  Could not remove binary automatically: {}", e);
            println!("   Please manually remove: {}", current_exe.display());
        } else {
            println!("✓ Removed TM CLI binary");
        }
    }

    #[cfg(windows)]
    {
        // On Windows, we need to use a different approach
        println!("⚠️  Windows detected - binary removal requires manual deletion");
        println!("   Please manually remove: {}", current_exe.display());
        println!("   Or run: del \"{}\"", current_exe.display());
    }

    println!("");
    println!("✅ TM CLI has been uninstalled successfully!");
    println!("   Thank you for using TM CLI!");

    Ok(())
}
