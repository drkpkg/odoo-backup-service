# Release Information

This document describes the release process and available packages for the Odoo Backup Service.

## Supported Distributions

The following distributions are officially supported with pre-built packages:

- **Ubuntu 22.04 LTS (Jammy Jellyfish)**
- **Ubuntu 24.04 LTS (Noble Numbat)**
- **Debian 12 (Bookworm)**

## Release Process

### Automatic Releases

Releases are automatically triggered when:

1. **Git Tags**: Push a tag starting with `v` (e.g., `v0.1.0`)
2. **Manual Trigger**: Use GitHub Actions workflow dispatch

### Creating a Release

#### Method 1: Using the Release Script

```bash
# Create a release with automatic version detection
./scripts/release.sh --version 0.1.0 --message "Initial release"

# Build only (no release)
./scripts/release.sh --build-only
```

#### Method 2: Manual Git Tag

```bash
# Update version in Cargo.toml
# Create and push tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

#### Method 3: GitHub Actions Manual Trigger

1. Go to Actions tab in GitHub
2. Select "Build and Release" workflow
3. Click "Run workflow"
4. Enter version (e.g., v0.1.0)
5. Click "Run workflow"

## Release Artifacts

Each release includes the following artifacts:

### Debian Packages (.deb)

- `odoo-backup-service_<version>-1_jammy_amd64.deb` - Ubuntu 22.04
- `odoo-backup-service_<version>-1_noble_amd64.deb` - Ubuntu 24.04
- `odoo-backup-service_<version>-1_bookworm_amd64.deb` - Debian 12

### Binary Archives (.tar.gz)

- `odoo-backup-service-ubuntu-22.04-amd64.tar.gz`
- `odoo-backup-service-ubuntu-24.04-amd64.tar.gz`
- `odoo-backup-service-debian-12-amd64.tar.gz`

### Checksums

- `checksums.txt` - SHA256 checksums for all artifacts

## Installation from Release

### Debian Package Installation

```bash
# Download the appropriate .deb file for your distribution
wget https://github.com/daniel-uremix/odoo-backup-service/releases/download/v0.1.0/odoo-backup-service_0.1.0-1_jammy_amd64.deb

# Install the package
sudo dpkg -i odoo-backup-service_0.1.0-1_jammy_amd64.deb

# Fix dependencies if needed
sudo apt-get install -f
```

### Binary Archive Installation

```bash
# Download the appropriate binary archive
wget https://github.com/daniel-uremix/odoo-backup-service/releases/download/v0.1.0/odoo-backup-service-ubuntu-22.04-amd64.tar.gz

# Extract the archive
tar -xzf odoo-backup-service-ubuntu-22.04-amd64.tar.gz

# Install manually
sudo cp odoo-backup /usr/bin/
sudo mkdir -p /etc/odoo-backup
sudo cp config.json.example /etc/odoo-backup/config.json
sudo mkdir -p /var/backups/odoo
```

## Local Package Building

### Build Debian Package Locally

```bash
# Install build dependencies
sudo apt-get install build-essential pkg-config libssl-dev devscripts debhelper

# Build the package
make deb

# Clean build artifacts
make clean-deb
```

### Build Binary Archive Locally

```bash
# Build release binary
make build

# Create release archive
make release-archive
```

## Package Contents

### Debian Package Contents

- `/usr/bin/odoo-backup` - Main executable
- `/etc/odoo-backup/config.json` - Configuration file
- `/var/backups/odoo/` - Backup directory
- `/usr/share/doc/odoo-backup-service/` - Documentation

### Binary Archive Contents

- `odoo-backup` - Main executable
- `README.md` - Project documentation
- `INSTALL.md` - Installation guide
- `TESTING.md` - Testing documentation
- `config.json.example` - Example configuration

## Dependencies

### Runtime Dependencies

- `docker.io` or `docker-ce` - Docker daemon
- `curl` - HTTP client for API calls

### Build Dependencies

- `rustc` - Rust compiler
- `cargo` - Rust package manager
- `build-essential` - Build tools
- `pkg-config` - Package configuration
- `libssl-dev` - OpenSSL development files
- `devscripts` - Debian development scripts
- `debhelper` - Debian helper scripts

## Version Management

### Version Format

Versions follow semantic versioning (SemVer): `MAJOR.MINOR.PATCH`

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Changelog

The changelog is automatically generated from:
- Git commits since last release
- Pull requests merged
- Issues closed

### Version Updates

To update the version:

1. Update `version` in `Cargo.toml`
2. Update `debian/changelog` if building locally
3. Create and push git tag
4. GitHub Actions will handle the rest

## Troubleshooting

### Package Installation Issues

```bash
# Check package dependencies
dpkg -I odoo-backup-service_*.deb

# Install missing dependencies
sudo apt-get update
sudo apt-get install docker.io curl

# Reinstall package
sudo dpkg -i odoo-backup-service_*.deb
```

### Binary Execution Issues

```bash
# Check binary permissions
ls -la /usr/bin/odoo-backup

# Fix permissions if needed
sudo chmod +x /usr/bin/odoo-backup

# Check dependencies
ldd /usr/bin/odoo-backup
```

### Configuration Issues

```bash
# Check configuration file
ls -la /etc/odoo-backup/config.json

# Verify configuration syntax
odoo-backup --config /etc/odoo-backup/config.json list
```

## Security

### Package Verification

All release artifacts are signed and include checksums for verification:

```bash
# Verify checksums
sha256sum -c checksums.txt

# Verify package signature (if available)
dpkg-sig --verify odoo-backup-service_*.deb
```

### Security Considerations

- Packages are built in isolated GitHub Actions runners
- All dependencies are pinned to specific versions
- No external network access during build process
- All artifacts are scanned for vulnerabilities

## Support

For issues with releases or packages:

1. Check the [Issues](https://github.com/daniel-uremix/odoo-backup-service/issues) page
2. Verify your distribution is supported
3. Check the troubleshooting section above
4. Create a new issue with detailed information
