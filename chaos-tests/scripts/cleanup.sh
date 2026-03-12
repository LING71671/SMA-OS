#!/bin/bash
# Cleanup script for chaos tests
# This script cleans up any resources created during chaos tests

set -e

echo "=== Cleaning up chaos test resources ==="

# Remove any temporary files
rm -rf /tmp/chaos-test-*

# Remove any leftover containers
docker ps -a | grep -E 'chaos|test' | awk '{print $1}' | xargs -r docker rm -f || true

# Remove any leftover networks
docker network ls | grep -E 'chaos|test' | awk '{print $1}' | xargs -r docker network rm || true

# Remove any leftover volumes
docker volume ls | grep -E 'chaos|test' | awk '{print $1}' | xargs -r docker volume rm || true

echo "Cleanup completed!"
