use std::fs;
use std::path::PathBuf;

pub fn get_data_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_dir().ok_or("could not determine data directory")?;

    let app_dir = data_dir.join("tm");

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }

    Ok(app_dir.join("tasks.json"))
}

pub fn get_data_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_dir().ok_or("could not determine data directory")?;
    Ok(data_dir.join("tm"))
}

pub fn format_path(path: &Vec<usize>) -> String {
    path.iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(".")
}
