use clap::Parser;
use log::{error, info, warn};
use std::collections::HashMap;
use std::env;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

mod backup;
mod cli;
mod config;
mod docker;
mod error;

use backup::BackupManager;
use cli::{Cli, Commands};
use config::Config;
use docker::DockerManager;
use error::Result;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    env::set_var("RUST_LOG", log_level);
    env_logger::init();

    info!("Starting Odoo Backup Service");

    if let Err(e) = run(cli).await {
        error!("Application error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Load configuration
    let config = Config::from_file(&cli.config)?;
    info!(
        "Loaded configuration with {} databases",
        config.databases.len()
    );

    let backup_manager = BackupManager::new(cli.backup_dir.clone());
    let docker_manager = DockerManager::new();

    match cli.command {
        Commands::Backup { client, parallel } => {
            if let Some(client_name) = client {
                // Backup specific client (parallel flag is ignored for single client)
                if let Some(db_config) = config.get_database(&client_name) {
                    info!("Backing up client: {}", client_name);
                    
                    // Create progress bar for single backup
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::default_spinner()
                            .template("{spinner:.green} {msg}")
                            .unwrap(),
                    );
                    pb.set_message(format!("Backing up {}...", client_name));
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));
                    
                    match backup_manager.backup_database(db_config).await {
                        Ok(backup_path) => {
                            pb.finish_with_message(format!("✓ Backup completed: {}", backup_path));
                            println!("Backup completed successfully: {}", backup_path);
                        }
                        Err(e) => {
                            pb.finish_with_message(format!("✗ Backup failed: {}", e));
                            error!("Backup failed for {}: {}", client_name, e);
                            return Err(e);
                        }
                    }
                } else {
                    error!(" Client '{}' not found in configuration", client_name);
                    return Err(error::BackupError::Config(format!(
                        "Client '{}' not found",
                        client_name
                    )));
                }
            } else {
                // Backup all clients
                if parallel {
                    info!("Backing up all configured databases in parallel");
                    
                    // Create multi-progress for parallel backups
                    let multi = MultiProgress::new();
                    let mut progress_bars = HashMap::new();

                    // Initialize progress bars for each database before starting backups
                    for db_config in &config.databases {
                        let pb = multi.add(ProgressBar::new_spinner());
                        pb.set_style(
                            ProgressStyle::default_spinner()
                                .template("{spinner:.green} {msg}")
                                .unwrap(),
                        );
                        pb.set_message(format!("Starting backup for {}...", db_config.name));
                        pb.enable_steady_tick(std::time::Duration::from_millis(100));
                        progress_bars.insert(db_config.name.clone(), pb);
                    }

                    // Start parallel backups
                    let results = backup_manager
                        .backup_all_databases_parallel(&config.databases)
                        .await?;

                    // Process results and update progress bars
                    let mut success_count = 0;
                    let mut failed_count = 0;
                    let mut success_results = Vec::new();
                    let mut failed_results = Vec::new();

                    for (name, result) in results {
                        if let Some(pb) = progress_bars.get(&name) {
                            match result {
                                Ok(backup_path) => {
                                    pb.finish_with_message(format!("✓ {}: {}", name, backup_path));
                                    success_count += 1;
                                    success_results.push((name.clone(), backup_path));
                                }
                                Err(e) => {
                                    pb.finish_with_message(format!("✗ {}: {}", name, e));
                                    failed_count += 1;
                                    failed_results.push((name.clone(), e.to_string()));
                                }
                            }
                        }
                    }

                    println!("\nBackup Summary:");
                    println!("  Successful: {}", success_count);
                    println!("  Failed: {}", failed_count);

                    if !success_results.is_empty() {
                        println!("\nSuccessful backups:");
                        for (client_name, backup_path) in &success_results {
                            println!("  - {}: {}", client_name, backup_path);
                        }
                    }

                    if !failed_results.is_empty() {
                        println!("\nFailed backups:");
                        for (client_name, error_msg) in &failed_results {
                            println!("  - {}: {}", client_name, error_msg);
                        }
                        warn!("Some backups failed");
                    }

                    if success_results.is_empty() {
                        return Err(error::BackupError::Unknown(
                            "No backups were completed successfully".to_string(),
                        ));
                    }
                } else {
                    info!("Backing up all configured databases sequentially");
                    let results = backup_manager
                        .backup_all_databases(&config.databases)
                        .await?;

                    if results.is_empty() {
                        warn!("No backups were completed successfully");
                    } else {
                        println!("Completed {} backups:", results.len());
                        for (client_name, backup_path) in results {
                            println!("  - {}: {}", client_name, backup_path);
                        }
                    }
                }
            }
        }
        Commands::List => {
            println!("Configured databases:");
            for (i, db) in config.databases.iter().enumerate() {
                println!("  {}. {} ({})", i + 1, db.name, db.database_name);
                println!("     Container: {}", db.container_name);
                println!("     URL: {}", db.url);
                println!("     Format: {}", db.backup_format);
                println!();
            }
        }
        Commands::Status => {
            println!("Docker container status:");
            let containers = docker_manager.list_containers().await?;

            for db in &config.databases {
                let is_running = docker_manager
                    .is_container_running(&db.container_name)
                    .await?;
                let status = if is_running { "Running" } else { "Stopped" };
                println!("  - {} ({}) - {}", db.name, db.container_name, status);
            }

            if containers.is_empty() {
                println!("  No containers are currently running");
            } else {
                println!("\nAll running containers:");
                for container in containers {
                    println!("  - {}", container);
                }
            }
        }
        Commands::Clean { client } => {
            if let Some(client_name) = client {
                // Clean specific client
                if let Some(db_config) = config.get_database(&client_name) {
                    info!("Cleaning old backups for client: {}", client_name);
                    let deleted_count = backup_manager.cleanup_old_backups(db_config).await?;
                    println!(
                        "Cleaned up {} old backup files for {}",
                        deleted_count, client_name
                    );
                } else {
                    error!(" Client '{}' not found in configuration", client_name);
                    return Err(error::BackupError::Config(format!(
                        "Client '{}' not found",
                        client_name
                    )));
                }
            } else {
                // Clean all clients
                info!("Cleaning old backups for all databases");
                let mut total_deleted = 0;
                for db_config in &config.databases {
                    let deleted_count = backup_manager.cleanup_old_backups(db_config).await?;
                    total_deleted += deleted_count;
                }
                println!("Cleaned up {} old backup files total", total_deleted);
            }
        }
        Commands::ListBackups { database } => {
            let backups = backup_manager.list_backups(database.as_deref()).await?;

            if backups.is_empty() {
                println!("No backup files found");
            } else {
                println!("Backup files:");
                for backup in backups {
                    println!("  - {}", backup);
                }
            }
        }
    }

    Ok(())
}
