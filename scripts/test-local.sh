#!/bin/bash

# Script simple para testear la construcción de Debian localmente
# Usa el Dockerfile.test para crear un entorno consistente

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

print_status "Building test Docker image..."
docker build -f Dockerfile.test -t odoo-backup-test .

print_status "Running Debian package build test..."

# Run the container and execute the build
docker run --rm -v "$PROJECT_ROOT:/workspace" -w /workspace odoo-backup-test bash -c "
    set -e
    
    echo '=== Building application ==='
    cargo build --release
    
    echo '=== Updating changelog ==='
    VERSION=\$(grep '^version = ' Cargo.toml | sed 's/version = \"\(.*\)\"/\1/')
    echo \"Version: \$VERSION\"
    sed -i \"s/0.1.0-1/\$VERSION-1/\" debian/changelog
    
    echo '=== Building Debian package ==='
    dpkg-buildpackage -us -uc -b -d
    
    echo '=== Listing generated files ==='
    ls -la ../odoo-backup-service_*
    
    echo '=== Copying .deb file to workspace ==='
    cp ../odoo-backup-service_*.deb .
    
    echo '=== Final workspace contents ==='
    ls -la
"

if [ $? -eq 0 ]; then
    print_status "Build completed successfully!"
    
    # Check if .deb file was created
    if [ -f "$PROJECT_ROOT/odoo-backup-service_0.1.0-1_amd64.deb" ]; then
        print_status "Generated .deb file:"
        ls -la "$PROJECT_ROOT/odoo-backup-service_0.1.0-1_amd64.deb"
    else
        print_warning "No .deb file found in expected location"
    fi
else
    print_error "Build failed!"
    exit 1
fi
