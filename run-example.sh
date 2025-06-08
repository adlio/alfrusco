#!/bin/bash

# Script to run alfrusco examples with mock Alfred environment variables
# Usage: ./run-example.sh <example_name> [args...]

if [ $# -eq 0 ]; then
    echo "Usage: $0 <example_name> [args...]"
    echo ""
    echo "Available examples:"
    echo "  static_output"
    echo "  success [--message \"Custom message\"]"
    echo "  random_user [search_term]"
    echo "  url_items"
    echo "  sleep [--duration-in-seconds N]"
    echo "  error [--file-path path]"
    echo "  async_success [--message \"Custom message\"]"
    echo "  async_error [--url URL] [--timeout N]"
    exit 1
fi

EXAMPLE_NAME=$1
shift  # Remove first argument, keep the rest

# Set up mock Alfred environment variables
export alfred_workflow_bundleid="com.example.alfrusco.${EXAMPLE_NAME}"
export alfred_workflow_cache="/tmp/alfrusco_cache_${EXAMPLE_NAME}"
export alfred_workflow_data="/tmp/alfrusco_data_${EXAMPLE_NAME}"
export alfred_version="5.0"
export alfred_version_build="2058"
export alfred_workflow_name="Alfrusco Example: ${EXAMPLE_NAME}"
export alfred_workflow_description="Example workflow demonstrating alfrusco features"
export alfred_workflow_version="1.0"
export alfred_debug="1"

# Create cache and data directories
mkdir -p "$alfred_workflow_cache"
mkdir -p "$alfred_workflow_data"

# Run the example
echo "Running example '$EXAMPLE_NAME' with mock Alfred environment..."
echo "Cache dir: $alfred_workflow_cache"
echo "Data dir: $alfred_workflow_data"
echo ""

cargo run --example "$EXAMPLE_NAME" -- "$@"