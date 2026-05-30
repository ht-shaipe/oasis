#!/bin/bash

# Exit on error
set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

function log_info() {
    echo -e "${GREEN}[INFO] $1${NC}"
}

function log_warn() {
    echo -e "${BLUE}[WARN] $1${NC}"
}

function log_error() {
    echo -e "${RED}[ERROR] $1${NC}"
}

# Check if bun is installed
if ! command -v bun &> /dev/null; then
    log_error "bun is not found. Please install bun package manager."
    exit 1
fi

BUILD_TYPE=${1:-"all"}

case $BUILD_TYPE in
    "web")
        log_info "Building frontend (web) only..."
        bun run build
        ;;
    "tauri")
        log_info "Building Tauri application..."
        bun run tauri:build
        ;;
    "all")
        log_info "Starting full build process..."
        # In tauri.conf.json, beforeBuildCommand is usually set to build the frontend.
        # We'll run bun run tauri:build which will trigger the frontend build automatically.
        log_info "Executing bun run tauri:build..."
        bun run tauri:build
        ;;
    *)
        log_error "Invalid argument: $BUILD_TYPE"
        echo "Usage: $0 [web|tauri|all]"
        exit 1
        ;;
esac

# Check for Tauri bundle output if applicable
if [[ "$BUILD_TYPE" == "tauri" || "$BUILD_TYPE" == "all" ]]; then
    BUNDLE_PATH="src-tauri/target/release/bundle"
    log_info "Checking for build artifacts in $BUNDLE_PATH..."
    
    if [ -d "$BUNDLE_PATH" ]; then
        log_info "Bundle directory found!"
        
        # Check for DMG files specifically
        DMG_FILES=$(find "$BUNDLE_PATH/dmg" -name "*.dmg" 2>/dev/null || true)
        if [ -n "$DMG_FILES" ]; then
            log_info "Found DMG package(s):"
            echo -e "${BLUE}$DMG_FILES${NC}"
        fi

        # Simple check for any files in the bundle directory
        if [ "$(ls -A $BUNDLE_PATH)" ]; then
            log_info "Generated bundle files summary:"
            find "$BUNDLE_PATH" -maxdepth 2 -not -path '*/.*'
        else
            log_warn "Bundle directory exists but appears to be empty."
        fi
    else
        log_error "Tauri bundle directory not found at $BUNDLE_PATH"
        log_error "The build might have failed or the output directory is different."
        exit 1
    fi
fi

log_info "Build process finished successfully."
