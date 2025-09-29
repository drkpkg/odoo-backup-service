# Makefile for Odoo Backup Service

# Variables
BINARY_NAME = odoo-backup
BINARY_PATH = target/release/$(BINARY_NAME)
INSTALL_BIN_DIR = /usr/bin
INSTALL_CONFIG_DIR = /etc/odoo-backup
INSTALL_CONFIG_FILE = $(INSTALL_CONFIG_DIR)/config.json
SAMPLE_CONFIG = config.json.example

# Default target
.PHONY: all
all: build

# Build the application
.PHONY: build
build:
	cargo build --release

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
	cargo clean

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

# Build Debian package
.PHONY: deb
deb: build
	@echo "Building Debian package..."
	dpkg-buildpackage -us -uc -b
	@echo "Debian package built successfully!"

# Clean Debian build artifacts
.PHONY: clean-deb
clean-deb:
	@echo "Cleaning Debian build artifacts..."
	rm -f ../odoo-backup-service_*.deb
	rm -f ../odoo-backup-service_*.dsc
	rm -f ../odoo-backup-service_*.tar.gz
	rm -f ../odoo-backup-service_*.changes
	rm -f ../odoo-backup-service_*.buildinfo
	@echo "Debian build artifacts cleaned!"

# Create release archive
.PHONY: release-archive
release-archive: build
	@echo "Creating release archive..."
	mkdir -p release
	cp target/release/odoo-backup-service release/odoo-backup
	cp README.md INSTALL.md TESTING.md release/
	cp config.json.example release/
	tar -czf odoo-backup-service-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m).tar.gz -C release .
	rm -rf release
	@echo "Release archive created successfully!"

# Show help
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build          - Build the application (release mode)"
	@echo "  dev            - Build the application (debug mode)"
	@echo "  test           - Run all tests"
	@echo "  clean          - Clean build artifacts"
	@echo "  install        - Install the application to system"
	@echo "  uninstall      - Remove the application from system"
	@echo "  status         - Show installation status"
	@echo "  run            - Install and run the application"
	@echo "  deb            - Build Debian package"
	@echo "  clean-deb      - Clean Debian build artifacts"
	@echo "  release-archive - Create release archive"
	@echo "  help           - Show this help message"
