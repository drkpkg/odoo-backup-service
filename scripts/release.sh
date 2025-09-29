#!/bin/bash

# Release script for Odoo Backup Service
# This script helps prepare and create releases

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to get version from Cargo.toml
get_version() {
    grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
}

# Function to check if we're in a git repository
check_git() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_error "Not in a git repository"
        exit 1
    fi
}

# Function to check if working directory is clean
check_clean_working_dir() {
    if ! git diff-index --quiet HEAD --; then
        print_error "Working directory is not clean. Please commit or stash changes."
        exit 1
    fi
}

# Function to check if tag exists
check_tag_exists() {
    local version=$1
    if git rev-parse "v$version" >/dev/null 2>&1; then
        print_error "Tag v$version already exists"
        exit 1
    fi
}

# Function to create release
create_release() {
    local version=$1
    local message=$2
    
    print_status "Creating release v$version"
    
    # Create and push tag
    git tag -a "v$version" -m "$message"
    git push origin "v$version"
    
    print_status "Release v$version created and pushed"
}

# Function to build release artifacts
build_artifacts() {
    print_status "Building release artifacts"
    
    # Run tests
    print_status "Running tests..."
    cargo test
    
    # Build release
    print_status "Building release binary..."
    cargo build --release
    
    # Create release archive
    print_status "Creating release archive..."
    make release-archive
    
    print_status "Release artifacts built successfully"
}

# Function to show help
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -v, --version VERSION    Version to release (e.g., 0.1.0)"
    echo "  -m, --message MESSAGE    Release message"
    echo "  -b, --build-only         Only build artifacts, don't create release"
    echo "  -h, --help               Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --version 0.1.0 --message 'Initial release'"
    echo "  $0 --build-only"
    echo "  $0 --version 0.1.1 --message 'Bug fixes'"
}

# Main function
main() {
    local version=""
    local message=""
    local build_only=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--version)
                version="$2"
                shift 2
                ;;
            -m|--message)
                message="$2"
                shift 2
                ;;
            -b|--build-only)
                build_only=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Check prerequisites
    if ! command_exists cargo; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    if ! command_exists git; then
        print_error "Git not found. Please install Git."
        exit 1
    fi
    
    # Check git repository
    check_git
    
    # Build artifacts
    build_artifacts
    
    # If build-only, exit here
    if [ "$build_only" = true ]; then
        print_status "Build-only mode. Release artifacts created."
        exit 0
    fi
    
    # If no version specified, get from Cargo.toml
    if [ -z "$version" ]; then
        version=$(get_version)
        print_warning "No version specified. Using version from Cargo.toml: $version"
    fi
    
    # If no message specified, use default
    if [ -z "$message" ]; then
        message="Release v$version"
        print_warning "No message specified. Using default: $message"
    fi
    
    # Check working directory is clean
    check_clean_working_dir
    
    # Check if tag already exists
    check_tag_exists "$version"
    
    # Create release
    create_release "$version" "$message"
    
    print_status "Release process completed successfully!"
    print_status "GitHub Actions will now build and publish the release artifacts."
}

# Run main function with all arguments
main "$@"
