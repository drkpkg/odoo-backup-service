use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "odoo-backup")]
#[command(about = "A Rust CLI application to automate Odoo backups inside Docker containers")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to the databases configuration file
    #[arg(short, long, default_value = "/etc/odoo-backup/config.json")]
    pub config: String,

    /// Host directory to store backups
    #[arg(short, long, default_value = "/var/backups/odoo")]
    pub backup_dir: String,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run backups for all configured databases
    Backup {
        /// Backup only a specific client by name
        #[arg(short, long)]
        client: Option<String>,
    },
    /// List all configured databases
    List,
    /// Check status of Docker containers
    Status,
    /// Clean old backup files
    Clean {
        /// Clean backups for a specific client by name
        #[arg(short, long)]
        client: Option<String>,
    },
    /// List existing backup files
    ListBackups {
        /// List backups for a specific database
        #[arg(short, long)]
        database: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing_backup_command() {
        let cli = Cli::try_parse_from(&["odoo-backup", "backup"]).unwrap();
        assert!(matches!(cli.command, Commands::Backup { client: None }));
        assert_eq!(cli.config, "/etc/odoo-backup/config.json");
        assert_eq!(cli.backup_dir, "/var/backups/odoo");
        assert!(!cli.verbose);
    }

    #[test]
    fn test_cli_parsing_backup_with_client() {
        let cli = Cli::try_parse_from(&["odoo-backup", "backup", "--client", "Test Client"]).unwrap();
        match cli.command {
            Commands::Backup { client } => {
                assert_eq!(client, Some("Test Client".to_string()));
            }
            _ => panic!("Expected Backup command"),
        }
    }

    #[test]
    fn test_cli_parsing_list_command() {
        let cli = Cli::try_parse_from(&["odoo-backup", "list"]).unwrap();
        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn test_cli_parsing_status_command() {
        let cli = Cli::try_parse_from(&["odoo-backup", "status"]).unwrap();
        assert!(matches!(cli.command, Commands::Status));
    }

    #[test]
    fn test_cli_parsing_clean_command() {
        let cli = Cli::try_parse_from(&["odoo-backup", "clean"]).unwrap();
        assert!(matches!(cli.command, Commands::Clean { client: None }));
    }

    #[test]
    fn test_cli_parsing_clean_with_client() {
        let cli = Cli::try_parse_from(&["odoo-backup", "clean", "--client", "Test Client"]).unwrap();
        match cli.command {
            Commands::Clean { client } => {
                assert_eq!(client, Some("Test Client".to_string()));
            }
            _ => panic!("Expected Clean command"),
        }
    }

    #[test]
    fn test_cli_parsing_list_backups_command() {
        let cli = Cli::try_parse_from(&["odoo-backup", "list-backups"]).unwrap();
        assert!(matches!(cli.command, Commands::ListBackups { database: None }));
    }

    #[test]
    fn test_cli_parsing_list_backups_with_database() {
        let cli = Cli::try_parse_from(&["odoo-backup", "list-backups", "--database", "test_db"]).unwrap();
        match cli.command {
            Commands::ListBackups { database } => {
                assert_eq!(database, Some("test_db".to_string()));
            }
            _ => panic!("Expected ListBackups command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_custom_config() {
        let cli = Cli::try_parse_from(&["odoo-backup", "-c", "custom.json", "list"]).unwrap();
        assert_eq!(cli.config, "custom.json");
        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn test_cli_parsing_with_custom_backup_dir() {
        let cli = Cli::try_parse_from(&["odoo-backup", "-b", "/custom/backups", "list"]).unwrap();
        assert_eq!(cli.backup_dir, "/custom/backups");
        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn test_cli_parsing_with_verbose() {
        let cli = Cli::try_parse_from(&["odoo-backup", "-v", "list"]).unwrap();
        assert!(cli.verbose);
        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn test_cli_parsing_with_short_flags() {
        let cli = Cli::try_parse_from(&["odoo-backup", "-c", "test.json", "-b", "/tmp", "-v", "backup"]).unwrap();
        assert_eq!(cli.config, "test.json");
        assert_eq!(cli.backup_dir, "/tmp");
        assert!(cli.verbose);
        assert!(matches!(cli.command, Commands::Backup { client: None }));
    }

    #[test]
    fn test_cli_parsing_with_long_flags() {
        let cli = Cli::try_parse_from(&["odoo-backup", "--config", "test.json", "--backup-dir", "/tmp", "--verbose", "backup"]).unwrap();
        assert_eq!(cli.config, "test.json");
        assert_eq!(cli.backup_dir, "/tmp");
        assert!(cli.verbose);
        assert!(matches!(cli.command, Commands::Backup { client: None }));
    }

    #[test]
    fn test_cli_help() {
        let cli = Cli::try_parse_from(&["odoo-backup", "--help"]);
        assert!(cli.is_err()); // Help causes early exit
    }

    #[test]
    fn test_cli_version() {
        let cli = Cli::try_parse_from(&["odoo-backup", "--version"]);
        assert!(cli.is_err()); // Version causes early exit
    }

    #[test]
    fn test_commands_enum_variants() {
        // Test that all command variants can be created
        let _backup = Commands::Backup { client: None };
        let _backup_with_client = Commands::Backup { client: Some("test".to_string()) };
        let _list = Commands::List;
        let _status = Commands::Status;
        let _clean = Commands::Clean { client: None };
        let _clean_with_client = Commands::Clean { client: Some("test".to_string()) };
        let _list_backups = Commands::ListBackups { database: None };
        let _list_backups_with_db = Commands::ListBackups { database: Some("test".to_string()) };
    }

    #[test]
    fn test_cli_default_values() {
        let cli = Cli::try_parse_from(&["odoo-backup", "list"]).unwrap();
        assert_eq!(cli.config, "/etc/odoo-backup/config.json");
        assert_eq!(cli.backup_dir, "/var/backups/odoo");
        assert!(!cli.verbose);
    }
}
