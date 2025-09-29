# Odoo Backup Service

A Rust CLI application to automate Odoo database backups inside Docker containers. This tool replaces the manual process of running `docker compose exec -it odoo-web curl ...` commands by providing a streamlined interface for managing multiple Odoo database backups.

## Features

- **Docker Integration**: Execute backups directly inside Odoo containers
- **Multi-Database Support**: Configure and backup multiple databases from a single JSON file
- **Automated Cleanup**: Automatic cleanup of old backup files based on retention policies
- **Status Monitoring**: Check container status and backup history
- **Error Handling**: Comprehensive error handling with detailed logging
- **Performance**: Built with Rust for fast and reliable execution

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Docker and Docker Compose
- Access to Odoo containers
- Make (for installation)

### System Installation

Install the application system-wide:

```bash
git clone <repository-url>
cd odoo-backup-service
make install
```

This will install:
- Binary: `/usr/bin/odoo-backup`
- Configuration: `/etc/odoo-backup/config.json`
- Backup directory: `/var/backups/odoo` (default)

### Pre-built Packages

Download pre-built packages for your distribution:

- **Ubuntu 22.04 (Jammy)**: `odoo-backup-service_*.deb`
- **Ubuntu 24.04 (Noble)**: `odoo-backup-service_*.deb`
- **Debian 12 (Bookworm)**: `odoo-backup-service_*.deb`
- **Binary Archives**: `odoo-backup-service-*-amd64.tar.gz`

Install from .deb package:
```bash
sudo dpkg -i odoo-backup-service_*.deb
sudo apt-get install -f  # Fix dependencies if needed
```

### Development Installation

For development or local testing:

```bash
git clone <repository-url>
cd odoo-backup-service
cargo build --release
```

The binary will be available at `target/release/odoo-backup-service`.

### Installation Management

```bash
# Check installation status
make status

# Uninstall the application
make uninstall

# Show available make targets
make help
```

## Configuration

### Database Configuration File

The application uses `/etc/odoo-backup/config.json` by default. You can edit this file with your Odoo database configurations:

```json
[
    {
        "name": "Client 1",
        "database_name": "client1_database",
        "url": "http://localhost:8069",
        "container_name": "client1_container",
        "master_password": "admin",
        "backup_format": "zip",
        "output_path": "/tmp/backups",
        "retention_days": 30
    },
    {
        "name": "Client 2",
        "database_name": "client2_database",
        "url": "http://localhost:8069",
        "container_name": "client2_container",
        "master_password": "admin",
        "backup_format": "zip",
        "output_path": "/tmp/backups",
        "retention_days": 30
    }
]
```

### Configuration Fields

| Field | Description | Required | Default |
|-------|-------------|----------|---------|
| `name` | Human-readable name for the database | Yes | - |
| `database_name` | Odoo database name | Yes | - |
| `url` | Odoo server URL | Yes | - |
| `container_name` | Docker container name | Yes | - |
| `master_password` | Odoo master password | Yes | - |
| `backup_format` | Backup format (zip/dump) | Yes | - |
| `output_path` | Path inside container for temporary backups | Yes | - |
| `retention_days` | Days to keep backup files | Yes | - |

## Usage

### Command Line Interface

```bash
odoo-backup-service [OPTIONS] <COMMAND>
```

### Global Options

- `-c, --config <CONFIG>`: Path to databases configuration file (default: `/etc/odoo-backup/config.json`)
- `-b, --backup-dir <BACKUP_DIR>`: Host directory to store backups (default: `/var/backups/odoo`)
- `-v, --verbose`: Enable verbose logging
- `-h, --help`: Print help information
- `-V, --version`: Print version information

### Commands

#### 1. Backup Databases

```bash
# Backup all configured databases
odoo-backup-service backup

# Backup a specific client
odoo-backup-service backup --client "Client 1"

# Use custom config and backup directory
odoo-backup-service -c my-config.json -b /path/to/backups backup
```

#### 2. List Configured Databases

```bash
odoo-backup-service list
```

#### 3. Check Container Status

```bash
odoo-backup-service status
```

#### 4. Clean Old Backups

```bash
# Clean old backups for all databases
odoo-backup-service clean

# Clean old backups for specific client
odoo-backup-service clean --client "Client 1"
```

#### 5. List Existing Backups

```bash
# List all backup files
odoo-backup-service list-backups

# List backups for specific database
odoo-backup-service list-backups --database "client1_database"
```

## How It Works

### Backup Process

1. **Container Check**: Verifies that the target Docker container is running
2. **Backup Execution**: Executes a `curl` command inside the container to trigger Odoo's backup API
3. **File Transfer**: Copies the backup file from the container to the host system
4. **Cleanup**: Removes temporary backup files from the container
5. **Retention**: Applies retention policy to clean up old backup files

### Docker Commands Used

The application executes the following Docker commands internally:

```bash
# Check container status
docker ps --filter name=container_name --format "{{.Names}}"

# Execute backup inside container
docker exec container_name curl -X POST \
  -F "master_pwd=password" \
  -F "name=database_name" \
  -F "backup_format=zip" \
  http://localhost:8069/web/database/backup \
  -o /tmp/backup_file.zip

# Copy backup to host
docker cp container_name:/tmp/backup_file.zip ./backups/
```

## Error Handling

The application provides comprehensive error handling for various scenarios:

- **Configuration Errors**: Invalid JSON, missing required fields
- **Docker Errors**: Container not running, permission issues
- **Network Errors**: Odoo API unavailable, connection timeouts
- **File System Errors**: Permission denied, disk space issues
- **Odoo API Errors**: Authentication failures, database locks

## Logging

The application uses structured logging with different levels:

- `INFO`: Successful operations and general information
- `WARN`: Non-critical issues and warnings
- `ERROR`: Failed operations and critical errors
- `DEBUG`: Detailed execution traces (use `-v` flag)

## Examples

### Basic Usage

```bash
# List all configured databases
odoo-backup list

# Check container status
odoo-backup status

# Run backup for all databases
odoo-backup backup

# Run backup for specific client with verbose logging
odoo-backup -v backup --client "Client 1"
```

### Advanced Usage

```bash
# Use custom configuration file
odoo-backup -c /path/to/config.json backup

# Store backups in custom directory
odoo-backup -b /var/backups/odoo backup

# Clean old backups and show verbose output
odoo-backup -v clean
```

## Security Considerations

- **Master Passwords**: Store master passwords securely, consider using environment variables
- **File Permissions**: Ensure backup files have appropriate permissions
- **Docker Access**: The application requires Docker access to execute commands inside containers
- **Network Security**: Ensure Odoo API endpoints are properly secured

## Troubleshooting

### Common Issues

1. **Container Not Running**
   ```
   ERROR: Container 'client1_container' is not running
   ```
   Solution: Start the container using `docker start client1_container`

2. **Permission Denied**
   ```
   ERROR: Failed to execute backup command: permission denied
   ```
   Solution: Ensure the user has Docker permissions or run with `sudo`

3. **Configuration Error**
   ```
   ERROR: Invalid JSON in config file
   ```
   Solution: Validate your `databases.json` file format

4. **Backup API Error**
   ```
   ERROR: Backup command failed: authentication failed
   ```
   Solution: Check master password and Odoo server status

### Debug Mode

Use the `-v` flag to enable verbose logging for detailed troubleshooting:

```bash
./odoo-backup-service -v backup --client "Client 1"
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:
1. Check the troubleshooting section
2. Review the logs with verbose mode
3. Open an issue on the repository
