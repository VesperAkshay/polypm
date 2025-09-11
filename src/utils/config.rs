// Configuration utilities

pub fn get_ppm_home_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".ppm-store")
}

pub fn get_project_config_path() -> std::path::PathBuf {
    std::path::PathBuf::from("project.toml")
}

pub fn get_lock_file_path() -> std::path::PathBuf {
    std::path::PathBuf::from("ppm.lock")
}
