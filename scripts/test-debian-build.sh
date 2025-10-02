#!/bin/bash

# Script para testear la construcción de paquetes Debian localmente usando Docker
# Esto simula el entorno de GitHub Actions para debuggear problemas

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

# Default values
DISTRO="ubuntu-24.04"
CLEANUP=true
INTERACTIVE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--distro)
            DISTRO="$2"
            shift 2
            ;;
        -c|--cleanup)
            CLEANUP=true
            shift
            ;;
        --no-cleanup)
            CLEANUP=false
            shift
            ;;
        -i|--interactive)
            INTERACTIVE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -d, --distro DISTRO    Distribution to test (default: ubuntu-24.04)"
            echo "                          Options: ubuntu-22.04, ubuntu-24.04, debian-12"
            echo "  -c, --cleanup          Clean up Docker container after test (default)"
            echo "  --no-cleanup           Keep Docker container for inspection"
            echo "  -i, --interactive       Run in interactive mode"
            echo "  -h, --help             Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 --distro ubuntu-24.04"
            echo "  $0 --interactive --no-cleanup"
            echo "  $0 --distro debian-12"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate distro
case $DISTRO in
    ubuntu-22.04|ubuntu-24.04|debian-12)
        ;;
    *)
        print_error "Invalid distro: $DISTRO"
        print_error "Valid options: ubuntu-22.04, ubuntu-24.04, debian-12"
        exit 1
        ;;
esac

# Get the base image
case $DISTRO in
    ubuntu-22.04)
        BASE_IMAGE="ubuntu:22.04"
        ;;
    ubuntu-24.04)
        BASE_IMAGE="ubuntu:24.04"
        ;;
    debian-12)
        BASE_IMAGE="debian:12"
        ;;
esac

print_status "Testing Debian package build for $DISTRO using $BASE_IMAGE"

# Get project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
print_debug "Project root: $PROJECT_ROOT"

# Create a unique container name
CONTAINER_NAME="odoo-backup-test-$(date +%s)"

print_status "Creating Docker container: $CONTAINER_NAME"

# Create and run the container
if [ "$INTERACTIVE" = true ]; then
    print_status "Starting interactive container..."
    docker run -it --name "$CONTAINER_NAME" \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        "$BASE_IMAGE" \
        bash
else
    print_status "Running automated test..."
    
    # Run the container with the build commands
    docker run --name "$CONTAINER_NAME" \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        "$BASE_IMAGE" \
        bash -c "
            set -e
            
            echo '=== Installing dependencies ==='
            apt-get update
            apt-get install -y build-essential pkg-config libssl-dev devscripts debhelper curl
            
            echo '=== Installing Rust ==='
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
            source \$HOME/.cargo/env
            
            echo '=== Building application ==='
            cargo build --release
            
            echo '=== Updating changelog ==='
            VERSION=\$(grep '^version = ' Cargo.toml | sed 's/version = \"\(.*\)\"/\1/')
            echo \"Version: \$VERSION\"
            sed -i \"s/0.1.0-1/\$VERSION-1/\" debian/changelog
            
            echo '=== Building Debian package ==='
            export PATH=\"\$HOME/.cargo/bin:\$PATH\"
            export CARGO_HOME=\"\$HOME/.cargo\"
            export RUSTUP_HOME=\"\$HOME/.rustup\"
            
            dpkg-buildpackage -us -uc -b -d
            
            echo '=== Listing generated files ==='
            ls -la ../odoo-backup-service_*
            
            echo '=== Copying .deb file to workspace ==='
            cp ../odoo-backup-service_*.deb .
            
            echo '=== Final workspace contents ==='
            ls -la
            
            echo '=== Test completed successfully! ==='
        "
    
    # Check if the build was successful
    if [ $? -eq 0 ]; then
        print_status "Build completed successfully!"
        
        # Copy the generated .deb file to the host
        print_status "Copying generated .deb file to host..."
        docker cp "$CONTAINER_NAME:/workspace/odoo-backup-service_0.1.0-1_amd64.deb" "$PROJECT_ROOT/" || true
        
        # List the generated file
        if [ -f "$PROJECT_ROOT/odoo-backup-service_0.1.0-1_amd64.deb" ]; then
            print_status "Generated .deb file:"
            ls -la "$PROJECT_ROOT/odoo-backup-service_0.1.0-1_amd64.deb"
        else
            print_warning "No .deb file found in expected location"
        fi
    else
        print_error "Build failed!"
    fi
fi

# Cleanup
if [ "$CLEANUP" = true ]; then
    print_status "Cleaning up Docker container..."
    docker rm "$CONTAINER_NAME" 2>/dev/null || true
else
    print_warning "Container $CONTAINER_NAME left running for inspection"
    print_warning "To clean up later, run: docker rm $CONTAINER_NAME"
fi

print_status "Test completed!"
