use serde::{Deserialize, Serialize};
use std::fs;
use crate::error::{BackupError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub database_name: String,
    pub url: String,
    pub container_name: String,
    pub master_password: String,
    pub backup_format: String,
    pub output_path: String,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub databases: Vec<DatabaseConfig>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| BackupError::FileSystem(format!("Failed to read config file {}: {}", path, e)))?;
        
        // Try to parse as direct array first, then as Config struct
        let databases: Vec<DatabaseConfig> = serde_json::from_str(&content)
            .map_err(|e| BackupError::Config(format!("Invalid JSON in config file: {}", e)))?;
        
        let config = Config { databases };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.databases.is_empty() {
            return Err(BackupError::Config("No databases configured".to_string()));
        }

        for (i, db) in self.databases.iter().enumerate() {
            if db.name.is_empty() {
                return Err(BackupError::Config(format!("Database {}: name cannot be empty", i)));
            }
            if db.database_name.is_empty() {
                return Err(BackupError::Config(format!("Database {}: database_name cannot be empty", i)));
            }
            if db.url.is_empty() {
                return Err(BackupError::Config(format!("Database {}: url cannot be empty", i)));
            }
            if db.container_name.is_empty() {
                return Err(BackupError::Config(format!("Database {}: container_name cannot be empty", i)));
            }
            if db.master_password.is_empty() {
                return Err(BackupError::Config(format!("Database {}: master_password cannot be empty", i)));
            }
            if !["zip", "dump"].contains(&db.backup_format.as_str()) {
                return Err(BackupError::Config(format!("Database {}: backup_format must be 'zip' or 'dump'", i)));
            }
        }

        Ok(())
    }

    pub fn get_database(&self, name: &str) -> Option<&DatabaseConfig> {
        self.databases.iter().find(|db| db.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_config() -> DatabaseConfig {
        DatabaseConfig {
            name: "Test Client".to_string(),
            database_name: "test_database".to_string(),
            url: "http://localhost:8069".to_string(),
            container_name: "test_container".to_string(),
            master_password: "admin".to_string(),
            backup_format: "zip".to_string(),
            output_path: "/tmp/backups".to_string(),
            retention_days: 30,
        }
    }

    fn create_test_configs() -> Vec<DatabaseConfig> {
        vec![
            create_test_config(),
            DatabaseConfig {
                name: "Test Client 2".to_string(),
                database_name: "test_database_2".to_string(),
                url: "http://localhost:8069".to_string(),
                container_name: "test_container_2".to_string(),
                master_password: "admin".to_string(),
                backup_format: "dump".to_string(),
                output_path: "/tmp/backups".to_string(),
                retention_days: 7,
            },
        ]
    }

    #[test]
    fn test_config_from_file_valid() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.json");
        
        let test_configs = create_test_configs();
        let json_content = serde_json::to_string_pretty(&test_configs).unwrap();
        fs::write(&config_path, json_content).unwrap();

        let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.databases.len(), 2);
        assert_eq!(config.databases[0].name, "Test Client");
        assert_eq!(config.databases[1].name, "Test Client 2");
    }

    #[test]
    fn test_config_from_file_invalid_json() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid_config.json");
        fs::write(&config_path, "invalid json content").unwrap();

        let result = Config::from_file(config_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Config(_)));
    }

    #[test]
    fn test_config_from_file_missing_file() {
        let result = Config::from_file("nonexistent_file.json");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::FileSystem(_)));
    }

    #[test]
    fn test_config_validation_empty_databases() {
        let config = Config { databases: vec![] };
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Config(_)));
    }

    #[test]
    fn test_config_validation_missing_name() {
        let mut config = create_test_config();
        config.name = String::new();
        let config = Config { databases: vec![config] };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Config(_)));
    }

    #[test]
    fn test_config_validation_missing_database_name() {
        let mut config = create_test_config();
        config.database_name = String::new();
        let config = Config { databases: vec![config] };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Config(_)));
    }

    #[test]
    fn test_config_validation_missing_url() {
        let mut config = create_test_config();
        config.url = String::new();
        let config = Config { databases: vec![config] };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Config(_)));
    }

    #[test]
    fn test_config_validation_missing_container_name() {
        let mut config = create_test_config();
        config.container_name = String::new();
        let config = Config { databases: vec![config] };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Config(_)));
    }

    #[test]
    fn test_config_validation_missing_master_password() {
        let mut config = create_test_config();
        config.master_password = String::new();
        let config = Config { databases: vec![config] };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Config(_)));
    }

    #[test]
    fn test_config_validation_invalid_backup_format() {
        let mut config = create_test_config();
        config.backup_format = "invalid_format".to_string();
        let config = Config { databases: vec![config] };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::Config(_)));
    }

    #[test]
    fn test_config_validation_valid_zip_format() {
        let mut config = create_test_config();
        config.backup_format = "zip".to_string();
        let config = Config { databases: vec![config] };
        
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validation_valid_dump_format() {
        let mut config = create_test_config();
        config.backup_format = "dump".to_string();
        let config = Config { databases: vec![config] };
        
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_database_existing() {
        let configs = create_test_configs();
        let config = Config { databases: configs };
        
        let found = config.get_database("Test Client");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Client");
    }

    #[test]
    fn test_get_database_nonexistent() {
        let configs = create_test_configs();
        let config = Config { databases: configs };
        
        let found = config.get_database("Nonexistent Client");
        assert!(found.is_none());
    }

    #[test]
    fn test_database_config_creation() {
        let config = create_test_config();
        assert_eq!(config.name, "Test Client");
        assert_eq!(config.database_name, "test_database");
        assert_eq!(config.url, "http://localhost:8069");
        assert_eq!(config.container_name, "test_container");
        assert_eq!(config.master_password, "admin");
        assert_eq!(config.backup_format, "zip");
        assert_eq!(config.output_path, "/tmp/backups");
        assert_eq!(config.retention_days, 30);
    }
}
