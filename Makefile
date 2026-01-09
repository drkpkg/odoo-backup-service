# Makefile for Odoo Backup Service

# Variables
BINARY_NAME = odoo-backup
BINARY_PATH = target/release/$(BINARY_NAME)
INSTALL_BIN_DIR = /usr/bin
INSTALL_CONFIG_DIR = /etc/odoo-backup
INSTALL_CONFIG_FILE = $(INSTALL_CONFIG_DIR)/config.json
SAMPLE_CONFIG = config.json.example
BUMP_VERSION_SCRIPT = scripts/bump-version.sh

# Default target
.PHONY: all
all: build

# Build the application
.PHONY: build
build:
	cargo build --release

# Version bumping targets
.PHONY: version-patch version-minor version-major version
version-patch:
	@$(BUMP_VERSION_SCRIPT) --type patch

version-minor:
	@$(BUMP_VERSION_SCRIPT) --type minor

version-major:
	@$(BUMP_VERSION_SCRIPT) --type major

version:
	@echo "Current version: $$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
	@echo ""
	@echo "Available version bump commands:"
	@echo "  make version-patch  - Bump patch version (0.1.2 -> 0.1.3)"
	@echo "  make version-minor  - Bump minor version (0.1.2 -> 0.2.0)"
	@echo "  make version-major  - Bump major version (0.1.2 -> 1.0.0)"
	@echo ""
	@echo "Or use the script directly:"
	@echo "  ./scripts/bump-version.sh --type patch"
	@echo "  ./scripts/bump-version.sh --version 0.2.0"
	@echo "  ./scripts/bump-version.sh --dry-run"

# Build for development
.PHONY: dev
dev:
	cargo build

# Run tests
.PHONY: test
test:
	cargo test

# Clean build artifacts
.PHONY: clean
clean:
	@if [ -f /root/.cargo/bin/cargo ]; then \
		/root/.cargo/bin/cargo clean; \
	elif command -v cargo >/dev/null 2>&1; then \
		cargo clean; \
	else \
		echo "Cargo not available, skipping clean"; \
	fi

# Install the application
.PHONY: install
install: build
	@echo "Installing Odoo Backup Service..."
	@echo "Creating directories..."
	sudo mkdir -p $(INSTALL_CONFIG_DIR)
	@echo "Installing binary to $(INSTALL_BIN_DIR)/$(BINARY_NAME)..."
	sudo cp $(BINARY_PATH) $(INSTALL_BIN_DIR)/$(BINARY_NAME)
	sudo chmod +x $(INSTALL_BIN_DIR)/$(BINARY_NAME)
	@echo "Installing configuration file to $(INSTALL_CONFIG_FILE)..."
	sudo cp $(SAMPLE_CONFIG) $(INSTALL_CONFIG_FILE)
	sudo chmod 644 $(INSTALL_CONFIG_FILE)
	@echo "Installation completed successfully!"
	@echo ""
	@echo "Usage:"
	@echo "  $(BINARY_NAME) --help"
	@echo "  $(BINARY_NAME) list"
	@echo "  $(BINARY_NAME) backup"
	@echo ""
	@echo "Configuration file: $(INSTALL_CONFIG_FILE)"

# Uninstall the application
.PHONY: uninstall
uninstall:
	@echo "Uninstalling Odoo Backup Service..."
	sudo rm -f $(INSTALL_BIN_DIR)/$(BINARY_NAME)
	sudo rm -f $(INSTALL_CONFIG_FILE)
	sudo rmdir $(INSTALL_CONFIG_DIR) 2>/dev/null || true
	@echo "Uninstallation completed!"

# Show installation status
.PHONY: status
status:
	@echo "Checking installation status..."
	@if [ -f $(INSTALL_BIN_DIR)/$(BINARY_NAME) ]; then \
		echo "Binary: $(INSTALL_BIN_DIR)/$(BINARY_NAME) - INSTALLED"; \
	else \
		echo "Binary: $(INSTALL_BIN_DIR)/$(BINARY_NAME) - NOT INSTALLED"; \
	fi
	@if [ -f $(INSTALL_CONFIG_FILE) ]; then \
		echo "Config: $(INSTALL_CONFIG_FILE) - INSTALLED"; \
	else \
		echo "Config: $(INSTALL_CONFIG_FILE) - NOT INSTALLED"; \
	fi

# Run the application with installed config
.PHONY: run
run: install
	$(BINARY_NAME) --config $(INSTALL_CONFIG_FILE) list

# Build Debian package using Docker (automatically increments patch version)
.PHONY: deb
deb:
	@echo "Building Debian package using Docker..."
	@if ! command -v docker >/dev/null 2>&1; then \
		echo "Error: Docker is not installed or not in PATH"; \
		exit 1; \
	fi
	@$(BUMP_VERSION_SCRIPT) --type patch >/dev/null 2>&1
	@echo "Building Docker image for Debian package with BuildKit..."
	@DOCKER_BUILDKIT=1 docker build -f Dockerfile.deb -t odoo-backup-service-deb:latest . || (echo "Error: Docker build failed"; exit 1)
	@echo "Running Debian package build in Docker container..."
	@mkdir -p ../debian-packages
	@docker run --rm \
		-v "$(PWD):/workspace" \
		-v "$(PWD)/../debian-packages:/output" \
		-w /workspace \
		odoo-backup-service-deb:latest \
		make deb-docker || (echo "Error: Docker build failed"; exit 1)
	@echo "Debian package built successfully!"
	@echo "Package location: ../debian-packages/"

# Internal target for building Debian package (called inside Docker)
.PHONY: deb-docker
deb-docker: build
	@echo "Building Debian package inside Docker..."
	dpkg-buildpackage -us -uc -b
	@echo "Moving Debian package files to output directory..."
	@mkdir -p /output
	@mv ../odoo-backup-service_*.deb /output/ 2>/dev/null || true
	@mv ../odoo-backup-service_*.dsc /output/ 2>/dev/null || true
	@mv ../odoo-backup-service_*.tar.gz /output/ 2>/dev/null || true
	@mv ../odoo-backup-service_*.changes /output/ 2>/dev/null || true
	@mv ../odoo-backup-service_*.buildinfo /output/ 2>/dev/null || true
	@$(MAKE) post-build

# Build with version bump (patch)
.PHONY: build-bump
build-bump: version-patch build
	@echo "Version bumped and build completed!"

# Post-build cleanup - removes temporary Debian build artifacts
.PHONY: post-build
post-build:
	@echo "Running post-build cleanup..."
	@rm -rf debian/.debhelper/ 2>/dev/null || true
	@rm -f debian/debhelper-build-stamp 2>/dev/null || true
	@rm -f debian/files 2>/dev/null || true
	@rm -f debian/*.substvars 2>/dev/null || true
	@rm -rf debian/odoo-backup-service/ 2>/dev/null || true
	@rm -rf usr/ 2>/dev/null || true
	@rm -rf etc/ 2>/dev/null || true
	@rm -rf var/ 2>/dev/null || true
	@echo "Post-build cleanup completed!"

# Clean Debian build artifacts (including parent directory and Docker output)
.PHONY: clean-deb
clean-deb:
	@echo "Cleaning Debian build artifacts..."
	rm -f ../odoo-backup-service_*.deb
	rm -f ../odoo-backup-service_*.dsc
	rm -f ../odoo-backup-service_*.tar.gz
	rm -f ../odoo-backup-service_*.changes
	rm -f ../odoo-backup-service_*.buildinfo
	rm -f *.deb
	@if [ -d ../debian-packages ]; then \
		rm -f ../debian-packages/odoo-backup-service_*.*; \
		echo "Cleaned Docker output directory"; \
	fi
	@$(MAKE) post-build
	@echo "Debian build artifacts cleaned!"

# Clean all (including Cargo build artifacts and Debian artifacts)
.PHONY: clean-all
clean-all: clean clean-deb
	@echo "All artifacts cleaned!"

# Create release archive
.PHONY: release-archive
release-archive: build
	@echo "Creating release archive..."
	mkdir -p release
	cp target/release/odoo-backup-service release/odoo-backup
	cp README.md release/
	cp config.json.example release/
	tar -czf odoo-backup-service-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m).tar.gz -C release .
	rm -rf release
	@echo "Release archive created successfully!"
	@$(MAKE) post-build

# Show help
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build           - Build the application (release mode)"
	@echo "  dev             - Build the application (debug mode)"
	@echo "  test            - Run all tests"
	@echo "  clean           - Clean Cargo build artifacts"
	@echo "  clean-deb       - Clean Debian build artifacts"
	@echo "  clean-all       - Clean all build artifacts"
	@echo "  post-build      - Clean temporary Debian build files"
	@echo "  install         - Install the application to system"
	@echo "  uninstall       - Remove the application from system"
	@echo "  status          - Show installation status"
	@echo "  run             - Install and run the application"
	@echo "  deb             - Build Debian package using Docker (auto-increments patch version)"
	@echo "  release-archive - Create release archive"
	@echo "  help            - Show this help message"
