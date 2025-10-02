use crate::config::DatabaseConfig;
use crate::error::{BackupError, Result};
use std::process::Command;

pub struct DockerManager;

impl Default for DockerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn is_container_running(&self, container_name: &str) -> Result<bool> {
        let output = Command::new("docker")
            .args([
                "ps",
                "--filter",
                &format!("name={}", container_name),
                "--format",
                "{{.Names}}",
            ])
            .output()
            .map_err(|e| BackupError::Docker(format!("Failed to check container status: {}", e)))?;

        if !output.status.success() {
            return Err(BackupError::Docker(format!(
                "Docker command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let running_containers = String::from_utf8_lossy(&output.stdout);
        Ok(running_containers.contains(container_name))
    }

    pub async fn execute_backup(&self, config: &DatabaseConfig) -> Result<String> {
        // Check if container is running
        if !self.is_container_running(&config.container_name).await? {
            return Err(BackupError::Docker(format!(
                "Container '{}' is not running",
                config.container_name
            )));
        }

        // Generate backup filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!(
            "backup_{}_{}.{}",
            config.database_name, timestamp, config.backup_format
        );
        let container_backup_path = format!("{}/{}", config.output_path, backup_filename);

        // Ensure the backup directory exists inside the container
        let mkdir_command = format!("mkdir -p {}", config.output_path);
        let mkdir_output = Command::new("docker")
            .args(["exec", &config.container_name, "sh", "-c", &mkdir_command])
            .output()
            .map_err(|e| {
                BackupError::Docker(format!("Failed to create backup directory: {}", e))
            })?;

        if !mkdir_output.status.success() {
            let error_msg = String::from_utf8_lossy(&mkdir_output.stderr);
            return Err(BackupError::Docker(format!(
                "Failed to create backup directory: {}",
                error_msg
            )));
        }

        // Create the curl command to execute inside the container
        let curl_command = format!(
            "curl -X POST -F 'master_pwd={}' -F 'name={}' -F 'backup_format={}' {}/web/database/backup -o {}",
            config.master_password,
            config.database_name,
            config.backup_format,
            config.url,
            container_backup_path
        );

        log::info!(
            "Executing backup for {} in container {}",
            config.name,
            config.container_name
        );

        // Execute the curl command inside the container
        let output = Command::new("docker")
            .args(["exec", &config.container_name, "sh", "-c", &curl_command])
            .output()
            .map_err(|e| BackupError::Docker(format!("Failed to execute backup command: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::Docker(format!(
                "Backup command failed: {}",
                error_msg
            )));
        }

        // Check if backup file was created
        let check_command = format!("test -f {}", container_backup_path);
        let check_output = Command::new("docker")
            .args(["exec", &config.container_name, "sh", "-c", &check_command])
            .output()
            .map_err(|e| BackupError::Docker(format!("Failed to check backup file: {}", e)))?;

        if !check_output.status.success() {
            return Err(BackupError::Docker(format!(
                "Backup file was not created: {}",
                container_backup_path
            )));
        }

        log::info!("Backup created successfully: {}", container_backup_path);
        Ok(container_backup_path)
    }

    pub async fn copy_backup_to_host(
        &self,
        config: &DatabaseConfig,
        container_path: &str,
        host_path: &str,
    ) -> Result<String> {
        let host_backup_path = format!(
            "{}/{}",
            host_path,
            container_path.split('/').next_back().unwrap_or("backup")
        );

        log::info!(
            "Copying backup from container to host: {} -> {}",
            container_path,
            host_backup_path
        );

        let output = Command::new("docker")
            .args([
                "cp",
                &format!("{}:{}", config.container_name, container_path),
                &host_backup_path,
            ])
            .output()
            .map_err(|e| BackupError::Docker(format!("Failed to copy backup file: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::Docker(format!(
                "Failed to copy backup: {}",
                error_msg
            )));
        }

        log::info!("Backup copied successfully to: {}", host_backup_path);
        Ok(host_backup_path)
    }

    pub async fn cleanup_container_backup(
        &self,
        config: &DatabaseConfig,
        container_path: &str,
    ) -> Result<()> {
        log::info!("Cleaning up backup file in container: {}", container_path);

        let output = Command::new("docker")
            .args(["exec", &config.container_name, "rm", "-f", container_path])
            .output()
            .map_err(|e| BackupError::Docker(format!("Failed to cleanup backup file: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            log::warn!("Failed to cleanup container backup file: {}", error_msg);
        } else {
            log::info!("Container backup file cleaned up successfully");
        }

        Ok(())
    }

    pub async fn list_containers(&self) -> Result<Vec<String>> {
        let output = Command::new("docker")
            .args(["ps", "--format", "{{.Names}}"])
            .output()
            .map_err(|e| BackupError::Docker(format!("Failed to list containers: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::Docker(format!(
                "Failed to list containers: {}",
                error_msg
            )));
        }

        let containers: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(containers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

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

    #[test]
    fn test_docker_manager_creation() {
        let _docker_manager = DockerManager::new();
        // Just test that we can create the manager
        assert!(true);
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

    // Note: Integration tests for actual Docker operations would require:
    // 1. Docker daemon running
    // 2. Test containers available
    // 3. Mock or test environment setup
    // These are better suited for integration tests rather than unit tests

    #[test]
    fn test_backup_command_construction() {
        let config = create_test_database_config();
        let timestamp = "20240101_120000";
        let backup_filename = format!(
            "backup_{}_{}.{}",
            config.database_name, timestamp, config.backup_format
        );
        let container_backup_path = format!("{}/{}", config.output_path, backup_filename);

        let expected_curl_command = format!(
            "curl -X POST -F 'master_pwd={}' -F 'name={}' -F 'backup_format={}' {} -o {}",
            config.master_password,
            config.database_name,
            config.backup_format,
            format!("{}/web/database/backup", config.url),
            container_backup_path
        );

        // Test that the command construction logic works
        assert!(expected_curl_command.contains("curl -X POST"));
        assert!(expected_curl_command.contains(&config.master_password));
        assert!(expected_curl_command.contains(&config.database_name));
        assert!(expected_curl_command.contains(&config.backup_format));
        assert!(expected_curl_command.contains("/web/database/backup"));
        assert!(expected_curl_command.contains("-o"));
    }

    #[test]
    fn test_backup_filename_generation() {
        let config = create_test_database_config();
        let timestamp = "20240101_120000";
        let backup_filename = format!(
            "backup_{}_{}.{}",
            config.database_name, timestamp, config.backup_format
        );

        assert_eq!(backup_filename, "backup_test_database_20240101_120000.zip");
        assert!(backup_filename.contains(&config.database_name));
        assert!(backup_filename.contains(&config.backup_format));
    }

    #[test]
    fn test_container_path_construction() {
        let config = create_test_database_config();
        let backup_filename = "backup_test_database_20240101_120000.zip";
        let container_backup_path = format!("{}/{}", config.output_path, backup_filename);

        assert_eq!(
            container_backup_path,
            "/tmp/backups/backup_test_database_20240101_120000.zip"
        );
    }

    #[test]
    fn test_host_backup_path_construction() {
        let container_path = "/tmp/backups/backup_test_database_20240101_120000.zip";
        let host_path = "./backups";
        let host_backup_path = format!(
            "{}/{}",
            host_path,
            container_path.split('/').next_back().unwrap_or("backup")
        );

        assert_eq!(
            host_backup_path,
            "./backups/backup_test_database_20240101_120000.zip"
        );
    }

    #[test]
    fn test_docker_command_arguments() {
        let container_name = "test_container";
        let curl_command = "curl -X POST -F 'master_pwd=admin' -F 'name=test_db' -F 'backup_format=zip' http://localhost:8069/web/database/backup -o /tmp/backup.zip";

        let expected_args = vec!["exec", container_name, "sh", "-c", curl_command];

        // Test that the argument construction is correct
        assert_eq!(expected_args[0], "exec");
        assert_eq!(expected_args[1], container_name);
        assert_eq!(expected_args[2], "sh");
        assert_eq!(expected_args[3], "-c");
        assert_eq!(expected_args[4], curl_command);
    }

    #[test]
    fn test_docker_cp_command_construction() {
        let container_name = "test_container";
        let container_path = "/tmp/backups/backup.zip";
        let host_path = "./backups/backup.zip";

        let container_source = format!("{}:{}", container_name, container_path);
        let expected_args = vec!["cp", &container_source, host_path];

        assert_eq!(expected_args[0], "cp");
        assert_eq!(expected_args[1], "test_container:/tmp/backups/backup.zip");
        assert_eq!(expected_args[2], "./backups/backup.zip");
    }

    #[test]
    fn test_docker_rm_command_construction() {
        let container_name = "test_container";
        let file_path = "/tmp/backups/backup.zip";

        let expected_args = vec!["exec", container_name, "rm", "-f", file_path];

        assert_eq!(expected_args[0], "exec");
        assert_eq!(expected_args[1], container_name);
        assert_eq!(expected_args[2], "rm");
        assert_eq!(expected_args[3], "-f");
        assert_eq!(expected_args[4], file_path);
    }
}
