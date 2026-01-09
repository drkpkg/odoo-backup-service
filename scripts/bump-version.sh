#!/bin/bash

# Version bump script for Odoo Backup Service
# Automatically increments version in Cargo.toml and debian/changelog

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

# Function to get current version from Cargo.toml
get_current_version() {
    grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
}

# Function to increment version
increment_version() {
    local version=$1
    local type=$2
    
    IFS='.' read -r -a parts <<< "$version"
    local major=${parts[0]}
    local minor=${parts[1]}
    local patch=${parts[2]}
    
    case $type in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            print_error "Invalid version type: $type. Use: major, minor, or patch"
            exit 1
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Function to update Cargo.toml
update_cargo_toml() {
    local new_version=$1
    local platform=$(uname)
    
    if [[ "$platform" == "Darwin" ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    fi
    
    print_status "Updated Cargo.toml to version $new_version"
}

# Function to update debian/changelog
update_debian_changelog() {
    local new_version=$1
    local old_version=$2
    local date=$(date -R)
    local user_name=$(git config user.name 2>/dev/null || echo "Unknown")
    local user_email=$(git config user.email 2>/dev/null || echo "unknown@example.com")
    
    # Create new changelog entry
    local changelog_entry="odoo-backup-service ($new_version-1) unstable; urgency=medium

  * Version bump to $new_version

 -- $user_name <$user_email>  $date

"
    
    # Prepend to changelog
    local platform=$(uname)
    if [[ "$platform" == "Darwin" ]]; then
        # macOS
        echo "$changelog_entry$(cat debian/changelog)" > debian/changelog.tmp
        mv debian/changelog.tmp debian/changelog
    else
        # Linux
        echo "$changelog_entry$(cat debian/changelog)" > debian/changelog.tmp
        mv debian/changelog.tmp debian/changelog
    fi
    
    print_status "Updated debian/changelog to version $new_version"
}

# Function to show help
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -t, --type TYPE    Version type to bump: major, minor, or patch (default: patch)"
    echo "  -v, --version VER  Set specific version (e.g., 0.2.0)"
    echo "  -d, --dry-run      Show what would be changed without making changes"
    echo "  -h, --help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                 # Bump patch version (0.1.2 -> 0.1.3)"
    echo "  $0 --type minor    # Bump minor version (0.1.2 -> 0.2.0)"
    echo "  $0 --type major    # Bump major version (0.1.2 -> 1.0.0)"
    echo "  $0 --version 0.2.0 # Set specific version"
    echo "  $0 --dry-run       # Preview changes"
}

# Main function
main() {
    local version_type="patch"
    local specific_version=""
    local dry_run=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--type)
                version_type="$2"
                shift 2
                ;;
            -v|--version)
                specific_version="$2"
                shift 2
                ;;
            -d|--dry-run)
                dry_run=true
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
    
    # Get current version
    local current_version=$(get_current_version)
    print_status "Current version: $current_version"
    
    # Determine new version
    local new_version
    if [ -n "$specific_version" ]; then
        new_version="$specific_version"
        print_status "Setting version to: $new_version"
    else
        new_version=$(increment_version "$current_version" "$version_type")
        print_status "Bumping $version_type version: $current_version -> $new_version"
    fi
    
    # Validate version format
    if ! [[ $new_version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        print_error "Invalid version format: $new_version. Expected format: X.Y.Z"
        exit 1
    fi
    
    if [ "$dry_run" = true ]; then
        print_warning "DRY RUN - No changes will be made"
        echo ""
        echo "Would update:"
        echo "  Cargo.toml: version = \"$current_version\" -> version = \"$new_version\""
        echo "  debian/changelog: Add new entry for $new_version-1"
        exit 0
    fi
    
    # Update files
    update_cargo_toml "$new_version"
    update_debian_changelog "$new_version" "$current_version"
    
    print_status "Version bumped successfully to $new_version"
    print_status "Files updated:"
    print_status "  - Cargo.toml"
    print_status "  - debian/changelog"
}

# Run main function with all arguments
main "$@"
