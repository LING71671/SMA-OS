#!/bin/bash
# Start all SMA-OS services

set -e

echo "Starting SMA-OS Services..."
echo "=========================="

# Export environment variables
export $(cat .env | xargs)
export DATABASE_URL="postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}"
export REDIS_URL="redis://localhost:6379"

echo ""
echo "1. Starting state-engine..."
cd control-plane
./target/release/state-engine &
STATE_ENGINE_PID=$!
echo "   state-engine PID: $STATE_ENGINE_PID"

echo ""
echo "2. Starting teardown-ctrl..."
./target/release/teardown-ctrl &
TEARDOWN_CTRL_PID=$!
echo "   teardown-ctrl PID: $TEARDOWN_CTRL_PID"

cd ..

echo ""
echo "3. Starting memory-bus ingestion..."
cd memory-bus
./bin/ingestion &
INGESTION_PID=$!
echo "   ingestion PID: $INGESTION_PID"

cd ..

echo ""
echo "4. Starting orchestration manager..."
cd orchestration
./bin/manager &
MANAGER_PID=$!
echo "   manager PID: $MANAGER_PID"

cd ..

echo ""
echo "=========================="
echo "Services started:"
echo "  - state-engine (PID: $STATE_ENGINE_PID)"
echo "  - teardown-ctrl (PID: $TEARDOWN_CTRL_PID)"
echo "  - ingestion (PID: $INGESTION_PID)"
echo "  - manager (PID: $MANAGER_PID)"
echo ""
echo "Press Ctrl+C to stop all services"
echo "=========================="

# Wait for all background processes
wait
