use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackupError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Docker error: {0}")]
    Docker(String),

    #[error("Network error: {0}")]
    #[allow(dead_code)]
    Network(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Odoo API error: {0}")]
    #[allow(dead_code)]
    OdooApi(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Unknown error: {0}")]
    #[allow(dead_code)]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, BackupError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_backup_error_display() {
        let config_error = BackupError::Config("Test config error".to_string());
        assert_eq!(
            format!("{}", config_error),
            "Configuration error: Test config error"
        );

        let docker_error = BackupError::Docker("Test docker error".to_string());
        assert_eq!(
            format!("{}", docker_error),
            "Docker error: Test docker error"
        );

        let file_error = BackupError::FileSystem("Test file error".to_string());
        assert_eq!(
            format!("{}", file_error),
            "File system error: Test file error"
        );
    }

    #[test]
    fn test_backup_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let backup_error: BackupError = io_error.into();

        match backup_error {
            BackupError::Io(e) => {
                assert_eq!(e.kind(), io::ErrorKind::NotFound);
                assert_eq!(e.to_string(), "File not found");
            }
            _ => panic!("Expected Io error variant"),
        }
    }

    #[test]
    fn test_backup_error_from_json_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let backup_error: BackupError = json_error.into();

        match backup_error {
            BackupError::Json(e) => {
                assert!(e.to_string().contains("expected"));
            }
            _ => panic!("Expected Json error variant"),
        }
    }

    #[tokio::test]
    async fn test_backup_error_from_reqwest_error() {
        // Create a reqwest error by trying to connect to an invalid URL
        let reqwest_error = reqwest::get("http://[::1]:99999").await.unwrap_err();
        let backup_error: BackupError = reqwest_error.into();

        match backup_error {
            BackupError::Http(e) => {
                assert!(!e.to_string().is_empty());
            }
            _ => panic!("Expected Http error variant"),
        }
    }

    #[test]
    fn test_backup_error_debug() {
        let error = BackupError::Config("Test error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("Test error"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<String> {
            Ok("success".to_string())
        }

        fn returns_error() -> Result<String> {
            Err(BackupError::Config("error".to_string()))
        }

        assert!(returns_result().is_ok());
        assert!(returns_error().is_err());
    }
}
