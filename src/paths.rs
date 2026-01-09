use directories::ProjectDirs;
use std::path::PathBuf;

pub fn get_wallet_data_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "nozy", "nozy") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir).ok();
        data_dir.to_path_buf()
    } else {
       
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE")) 
            .unwrap_or_else(|_| ".".to_string());
        
        let fallback = PathBuf::from(&home_dir).join(".nozy").join("data");
        std::fs::create_dir_all(&fallback).ok();
        fallback
    }
}


pub fn get_wallet_config_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "nozy", "nozy") {
        let config_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(config_dir).ok();
        config_dir.to_path_buf()
    } else {
       
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE")) 
            .unwrap_or_else(|_| ".".to_string());
        
        let fallback = PathBuf::from(&home_dir).join(".nozy").join("config");
        std::fs::create_dir_all(&fallback).ok();
        fallback
    }
}

pub fn get_wallet_data_path() -> PathBuf {
    get_wallet_data_dir()
}

pub fn get_wallet_config_path() -> PathBuf {
    get_wallet_config_dir().join("config.json")
}

/// Get the Zeaking index directory path
pub fn get_zeaking_index_dir() -> PathBuf {
    get_wallet_data_dir().join("zeaking")
}

