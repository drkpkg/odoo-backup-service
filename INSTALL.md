# Installation Guide

This guide explains how to install the Odoo Backup Service on your system.

## Quick Installation

```bash
git clone <repository-url>
cd odoo-backup-service
make install
```

## What Gets Installed

The `make install` command will install:

- **Binary**: `/usr/bin/odoo-backup`
- **Configuration**: `/etc/odoo-backup/config.json`
- **Backup Directory**: `/var/backups/odoo` (default)

## Prerequisites

Before installing, ensure you have:

- Rust 1.70+ installed
- Docker and Docker Compose
- Make utility
- sudo privileges (for system installation)

## Installation Steps

### 1. Clone the Repository

```bash
git clone <repository-url>
cd odoo-backup-service
```

### 2. Install the Application

```bash
make install
```

This will:
- Build the application in release mode
- Create `/etc/odoo-backup/` directory
- Install the binary to `/usr/bin/odoo-backup`
- Copy the example configuration to `/etc/odoo-backup/config.json`
- Set appropriate permissions

### 3. Configure the Application

Edit the configuration file:

```bash
sudo nano /etc/odoo-backup/config.json
```

Update the configuration with your actual Odoo database details:

```json
[
    {
        "name": "Production Database",
        "database_name": "prod_database",
        "url": "http://localhost:8069",
        "container_name": "odoo_prod_container",
        "master_password": "your_actual_master_password",
        "backup_format": "zip",
        "output_path": "/tmp/backups",
        "retention_days": 30
    }
]
```

### 4. Test the Installation

```bash
# Check installation status
make status

# Test the application
odoo-backup list
odoo-backup status
```

## Usage After Installation

Once installed, you can use the application from anywhere:

```bash
# List configured databases
odoo-backup list

# Check container status
odoo-backup status

# Run backups
odoo-backup backup

# Run backup for specific client
odoo-backup backup --client "Production Database"

# Clean old backups
odoo-backup clean
```

## Management Commands

### Check Installation Status

```bash
make status
```

### Uninstall the Application

```bash
make uninstall
```

This will remove:
- `/usr/bin/odoo-backup`
- `/etc/odoo-backup/config.json`
- `/etc/odoo-backup/` directory (if empty)

### Show Available Commands

```bash
make help
```

## Configuration Options

### Default Paths

- **Config File**: `/etc/odoo-backup/config.json`
- **Backup Directory**: `/var/backups/odoo`
- **Binary**: `/usr/bin/odoo-backup`

### Custom Paths

You can override the default paths:

```bash
# Use custom config file
odoo-backup -c /path/to/custom/config.json list

# Use custom backup directory
odoo-backup -b /custom/backup/path backup
```

## Security Considerations

### File Permissions

The installation sets appropriate permissions:
- Binary: `755` (executable by all users)
- Config: `644` (readable by all users, writable by root)
- Backup directory: `755` (accessible by all users)

### Master Passwords

**Important**: Update the master passwords in `/etc/odoo-backup/config.json` with your actual Odoo master passwords before running backups.

### Backup Directory

The default backup directory `/var/backups/odoo` is accessible by all users. Consider:
- Setting up proper backup rotation
- Monitoring disk space
- Implementing backup encryption if needed

## Troubleshooting

### Permission Issues

If you encounter permission issues:

```bash
# Check file permissions
ls -la /usr/bin/odoo-backup
ls -la /etc/odoo-backup/config.json

# Fix permissions if needed
sudo chmod +x /usr/bin/odoo-backup
sudo chmod 644 /etc/odoo-backup/config.json
```

### Configuration Issues

If the application can't find the config file:

```bash
# Check if config file exists
ls -la /etc/odoo-backup/config.json

# Test with explicit config path
odoo-backup -c /etc/odoo-backup/config.json list
```

### Docker Issues

Ensure Docker is running and containers are accessible:

```bash
# Check Docker status
docker ps

# Test container access
odoo-backup status
```

## Development Installation

For development purposes, you can build and run without installing:

```bash
# Build in development mode
make dev

# Run tests
make test

# Run the application directly
./target/debug/odoo-backup-service list
```

## Updating the Application

To update an existing installation:

```bash
# Pull latest changes
git pull

# Rebuild and reinstall
make install
```

## Uninstallation

To completely remove the application:

```bash
make uninstall
```

This will remove all installed files and directories.
