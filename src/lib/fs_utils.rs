// File system utilities

use std::path::Path;

pub fn ensure_directory_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty() 
        && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && !name.starts_with('-') 
        && !name.ends_with('-')
}
