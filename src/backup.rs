use crate::config::DatabaseConfig;
use crate::docker::DockerManager;
use crate::error::{BackupError, Result};
use chrono::{DateTime, Duration, Utc};
use std::fs;
use std::path::Path;
use tokio::task::JoinHandle;

pub struct BackupManager {
    docker: DockerManager,
    host_backup_dir: String,
}

impl BackupManager {
    pub fn new(host_backup_dir: String) -> Self {
        Self {
            docker: DockerManager::new(),
            host_backup_dir,
        }
    }

    pub async fn backup_database(&self, config: &DatabaseConfig) -> Result<String> {
        log::info!("Starting backup for database: {}", config.name);

        // Get individual backup directory for this database
        let db_backup_dir = self.get_database_backup_dir(config);

        // Ensure host backup directory exists
        self.ensure_backup_directory(&db_backup_dir).await?;

        // Execute backup inside container
        let container_backup_path = self.docker.execute_backup(config).await?;

        // Copy backup to host (using individual directory)
        let host_backup_path = self
            .docker
            .copy_backup_to_host(config, &container_backup_path, &db_backup_dir)
            .await?;

        // Cleanup container backup file
        self.docker
            .cleanup_container_backup(config, &container_backup_path)
            .await?;

        log::info!(
            "Backup completed successfully for {}: {}",
            config.name,
            host_backup_path
        );
        Ok(host_backup_path)
    }

    /// Get the individual backup directory path for a specific database
    fn get_database_backup_dir(&self, config: &DatabaseConfig) -> String {
        format!("{}/{}", self.host_backup_dir, config.database_name)
    }

    pub async fn backup_all_databases(
        &self,
        configs: &[DatabaseConfig],
    ) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for config in configs {
            match self.backup_database(config).await {
                Ok(backup_path) => {
                    results.push((config.name.clone(), backup_path));
                }
                Err(e) => {
                    let error_msg = format!("Failed to backup {}: {}", config.name, e);
                    log::error!("{}", error_msg);
                    errors.push(error_msg);
                }
            }
        }

        if !errors.is_empty() {
            log::warn!("Some backups failed: {}", errors.join(", "));
        }

        Ok(results)
    }

    /// Backup all databases in parallel using async tasks
    pub async fn backup_all_databases_parallel(
        &self,
        configs: &[DatabaseConfig],
    ) -> Result<Vec<(String, Result<String>)>> {
        let mut handles: Vec<JoinHandle<(String, Result<String>)>> = Vec::new();

        // Spawn a task for each database backup
        for config in configs {
            let config_clone = config.clone();
            let backup_manager = BackupManager {
                docker: DockerManager::new(),
                host_backup_dir: self.host_backup_dir.clone(),
            };

            let handle = tokio::spawn(async move {
                let name = config_clone.name.clone();
                let result = backup_manager.backup_database(&config_clone).await;
                (name, result)
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete and collect results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok((name, result)) => {
                    results.push((name, result));
                }
                Err(e) => {
                    log::error!("Task join error: {}", e);
                    results.push(("unknown".to_string(), Err(BackupError::Unknown(format!("Task join error: {}", e)))));
                }
            }
        }

        Ok(results)
    }

    pub async fn cleanup_old_backups(&self, config: &DatabaseConfig) -> Result<u32> {
        // Use individual database directory
        let db_backup_dir = self.get_database_backup_dir(config);
        let backup_dir = Path::new(&db_backup_dir);
        
        if !backup_dir.exists() {
            return Ok(0);
        }

        let retention_days = Duration::days(config.retention_days as i64);
        let cutoff_date = Utc::now() - retention_days;
        let mut deleted_count = 0;

        let entries = fs::read_dir(backup_dir).map_err(|e| {
            BackupError::FileSystem(format!("Failed to read backup directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                BackupError::FileSystem(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_file() {
                let metadata = entry.metadata().map_err(|e| {
                    BackupError::FileSystem(format!("Failed to get file metadata: {}", e))
                })?;

                let modified_time = metadata.modified().map_err(|e| {
                    BackupError::FileSystem(format!(
                        "Failed to get file modification time: {}",
                        e
                    ))
                })?;

                let modified_datetime: DateTime<Utc> = modified_time.into();

                if modified_datetime < cutoff_date {
                    log::info!("Deleting old backup: {}", path.display());
                    fs::remove_file(&path).map_err(|e| {
                        BackupError::FileSystem(format!("Failed to delete old backup: {}", e))
                    })?;
                    deleted_count += 1;
                }
            }
        }

        log::info!(
            "Cleaned up {} old backup files for {}",
            deleted_count,
            config.name
        );
        Ok(deleted_count)
    }

    async fn ensure_backup_directory(&self, backup_dir: &str) -> Result<()> {
        let backup_path = Path::new(backup_dir);
        if !backup_path.exists() {
            fs::create_dir_all(backup_path).map_err(|e| {
                BackupError::FileSystem(format!("Failed to create backup directory: {}", e))
            })?;
            log::info!("Created backup directory: {}", backup_dir);
        }
        Ok(())
    }

    pub async fn list_backups(&self, database_name: Option<&str>) -> Result<Vec<String>> {
        let backup_dir = Path::new(&self.host_backup_dir);
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        if let Some(db_name) = database_name {
            // List backups for a specific database (from its individual directory)
            let db_backup_dir = format!("{}/{}", self.host_backup_dir, db_name);
            let db_path = Path::new(&db_backup_dir);
            
            if db_path.exists() {
                let entries = fs::read_dir(db_path).map_err(|e| {
                    BackupError::FileSystem(format!("Failed to read backup directory: {}", e))
                })?;

                for entry in entries {
                    let entry = entry.map_err(|e| {
                        BackupError::FileSystem(format!("Failed to read directory entry: {}", e))
                    })?;
                    let path = entry.path();

                    if path.is_file() {
                        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        backups.push(format!("{}/{}", db_name, filename));
                    }
                }
            }
        } else {
            // List all backups from all database directories
            let entries = fs::read_dir(backup_dir).map_err(|e| {
                BackupError::FileSystem(format!("Failed to read backup directory: {}", e))
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    BackupError::FileSystem(format!("Failed to read directory entry: {}", e))
                })?;
                let path = entry.path();

                if path.is_dir() {
                    // This is a database directory
                    let db_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let db_entries = fs::read_dir(&path).map_err(|e| {
                        BackupError::FileSystem(format!("Failed to read database directory: {}", e))
                    })?;

                    for db_entry in db_entries {
                        let db_entry = db_entry.map_err(|e| {
                            BackupError::FileSystem(format!("Failed to read directory entry: {}", e))
                        })?;
                        let backup_path = db_entry.path();

                        if backup_path.is_file() {
                            let filename = backup_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            backups.push(format!("{}/{}", db_name, filename));
                        }
                    }
                }
            }
        }

        backups.sort();
        Ok(backups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;
    use tempfile::tempdir;

    fn create_test_database_config() -> DatabaseConfig {
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

    fn create_test_backup_manager() -> BackupManager {
        BackupManager::new("./test_backups".to_string())
    }

    #[test]
    fn test_backup_manager_creation() {
        let backup_manager = create_test_backup_manager();
        assert_eq!(backup_manager.host_backup_dir, "./test_backups");
    }

    #[test]
    fn test_database_config_creation() {
        let config = create_test_database_config();
        assert_eq!(config.name, "Test Client");
        assert_eq!(config.database_name, "test_database");
        assert_eq!(config.url, "http://localhost:8069");
        assert_eq!(config.container_name, "test_container");
        assert_eq!(config.master_password, "admin");
        assert_eq!(config.backup_format, "zip");
        assert_eq!(config.output_path, "/tmp/backups");
        assert_eq!(config.retention_days, 30);
    }

    #[test]
    fn test_backup_filename_pattern() {
        let config = create_test_database_config();
        let timestamp = "20240101_120000";
        let backup_filename = format!(
            "backup_{}_{}.{}",
            config.database_name, timestamp, config.backup_format
        );

        assert_eq!(backup_filename, "backup_test_database_20240101_120000.zip");
        assert!(backup_filename.contains(&config.database_name));
        assert!(backup_filename.contains(&config.backup_format));
        assert!(backup_filename.contains(timestamp));
    }

    #[test]
    fn test_backup_path_construction() {
        let config = create_test_database_config();
        let backup_filename = "backup_test_database_20240101_120000.zip";
        let container_backup_path = format!("{}/{}", config.output_path, backup_filename);

        assert_eq!(
            container_backup_path,
            "/tmp/backups/backup_test_database_20240101_120000.zip"
        );
    }

    #[test]
    fn test_retention_days_calculation() {
        let config = create_test_database_config();
        let retention_days = Duration::days(config.retention_days as i64);

        assert_eq!(retention_days.num_days(), 30);
    }

    #[test]
    fn test_backup_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let backup_dir = temp_dir.path().join("test_backups");
        let _backup_manager = BackupManager::new(backup_dir.to_string_lossy().to_string());

        // Test that the directory creation logic works
        assert!(!backup_dir.exists());
        // The actual directory creation would be tested in integration tests
    }

    #[test]
    fn test_backup_file_filtering() {
        let config = create_test_database_config();
        let filename1 = "backup_test_database_20240101_120000.zip";
        let filename2 = "backup_other_database_20240101_120000.zip";
        let filename3 = "backup_test_database_20240102_120000.zip";

        // Test filtering logic
        assert!(filename1.contains(&config.database_name));
        assert!(!filename2.contains(&config.database_name));
        assert!(filename3.contains(&config.database_name));
    }

    #[test]
    fn test_backup_file_extension() {
        let config = create_test_database_config();
        let backup_filename = format!("backup_{}.{}", config.database_name, config.backup_format);

        assert!(backup_filename.ends_with(&config.backup_format));
        assert!(backup_filename.ends_with(".zip"));
    }

    #[test]
    fn test_multiple_database_configs() {
        let configs = vec![
            create_test_database_config(),
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
        ];

        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].backup_format, "zip");
        assert_eq!(configs[1].backup_format, "dump");
        assert_eq!(configs[0].retention_days, 30);
        assert_eq!(configs[1].retention_days, 7);
    }

    #[test]
    fn test_backup_manager_with_custom_directory() {
        let custom_dir = "/custom/backup/path";
        let backup_manager = BackupManager::new(custom_dir.to_string());

        assert_eq!(backup_manager.host_backup_dir, custom_dir);
    }

    #[test]
    fn test_retention_policy_different_formats() {
        let zip_config = DatabaseConfig {
            name: "ZIP Client".to_string(),
            database_name: "zip_database".to_string(),
            url: "http://localhost:8069".to_string(),
            container_name: "zip_container".to_string(),
            master_password: "admin".to_string(),
            backup_format: "zip".to_string(),
            output_path: "/tmp/backups".to_string(),
            retention_days: 30,
        };

        let dump_config = DatabaseConfig {
            name: "DUMP Client".to_string(),
            database_name: "dump_database".to_string(),
            url: "http://localhost:8069".to_string(),
            container_name: "dump_container".to_string(),
            master_password: "admin".to_string(),
            backup_format: "dump".to_string(),
            output_path: "/tmp/backups".to_string(),
            retention_days: 7,
        };

        assert_eq!(zip_config.retention_days, 30);
        assert_eq!(dump_config.retention_days, 7);
        assert_eq!(zip_config.backup_format, "zip");
        assert_eq!(dump_config.backup_format, "dump");
    }

    // Note: Integration tests for actual backup operations would require:
    // 1. Docker daemon running
    // 2. Test containers available
    // 3. Mock or test environment setup
    // These are better suited for integration tests rather than unit tests
}
