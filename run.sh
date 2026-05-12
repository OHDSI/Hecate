#!/bin/bash
set -e

echo "======================================"
echo "Starting Hecate Containers"
echo "======================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Network name
NETWORK_NAME="hecate-network"

# Container names
DB_CONTAINER="hecate-db"
QDRANT_CONTAINER="qdrant"
API_CONTAINER="hecate-api"

# Check required environment variables
if [ -z "$OPENAI_API_KEY" ]; then
    echo -e "${YELLOW}WARNING: OPENAI_API_KEY not set${NC}"
    echo "The API will not be able to embed unknown search queries."
    echo "Set it with: export OPENAI_API_KEY=your-key-here"
    echo ""
fi

if [ -z "$UMLS_API_KEY" ]; then
    echo -e "${YELLOW}WARNING: UMLS_API_KEY not set${NC}"
    echo "The /api/concepts/{id}/definition endpoint will not work."
    echo ""
fi

# Create network if it doesn't exist
if ! docker network inspect $NETWORK_NAME >/dev/null 2>&1; then
    echo "Creating Docker network: $NETWORK_NAME"
    docker network create $NETWORK_NAME
    echo -e "${GREEN}✓ Network created${NC}"
else
    echo -e "${BLUE}Network $NETWORK_NAME already exists${NC}"
fi
echo ""

# Stop and remove existing containers if running
echo "Cleaning up existing containers..."
docker stop $DB_CONTAINER $QDRANT_CONTAINER $API_CONTAINER 2>/dev/null || true
docker rm $DB_CONTAINER $QDRANT_CONTAINER $API_CONTAINER 2>/dev/null || true
echo -e "${GREEN}✓ Cleanup complete${NC}"
echo ""

# Start PostgreSQL
echo "Starting PostgreSQL..."
docker run -d \
    --name $DB_CONTAINER \
    --network $NETWORK_NAME \
    -p 5432:5432 \
    -v hecate-postgres-data:/var/lib/postgresql/data \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGRES_DB=hecate \
    -e VOCAB_SCHEMA=vocab_27_feb_2026 \
    hecate-postgres:latest

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ PostgreSQL container started${NC}"
else
    echo -e "${RED}✗ Failed to start PostgreSQL${NC}"
    exit 1
fi
echo ""

# Start Qdrant
echo "Starting Qdrant..."
docker run -d \
    --name $QDRANT_CONTAINER \
    --network $NETWORK_NAME \
    -p 6333:6333 \
    -p 6334:6334 \
    -v hecate-qdrant-storage:/qdrant/storage \
    hecate-qdrant:latest

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Qdrant container started${NC}"
else
    echo -e "${RED}✗ Failed to start Qdrant${NC}"
    exit 1
fi
echo ""

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
MAX_RETRIES=30
RETRY_COUNT=0
until docker exec $DB_CONTAINER pg_isready -U postgres >/dev/null 2>&1 || [ $RETRY_COUNT -eq $MAX_RETRIES ]; do
    RETRY_COUNT=$((RETRY_COUNT+1))
    echo -n "."
    sleep 1
done
echo ""

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo -e "${RED}✗ PostgreSQL failed to start within 30 seconds${NC}"
    echo "Check logs with: docker logs $DB_CONTAINER"
    exit 1
fi
echo -e "${GREEN}✓ PostgreSQL is ready${NC}"
echo ""

# Wait for Qdrant to be ready
echo "Waiting for Qdrant to be ready..."
RETRY_COUNT=0
until curl -s http://localhost:6333/healthz >/dev/null 2>&1 || [ $RETRY_COUNT -eq $MAX_RETRIES ]; do
    RETRY_COUNT=$((RETRY_COUNT+1))
    echo -n "."
    sleep 1
done
echo ""

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo -e "${RED}✗ Qdrant failed to start within 30 seconds${NC}"
    echo "Check logs with: docker logs $QDRANT_CONTAINER"
    exit 1
fi
echo -e "${GREEN}✓ Qdrant is ready${NC}"
echo ""

# Start Hecate API
echo "Starting Hecate API..."
docker run -d \
    --name $API_CONTAINER \
    --network $NETWORK_NAME \
    -p 8080:8080 \
    -e SERVER_ADDR=0.0.0.0:8080 \
    -e QDRANT_URI=http://qdrant:6334 \
    -e VECTORDB_DATA_PATH=/app/all_pairs.txt \
    -e CORS_ORIGINS=http://localhost:5173 \
    -e PG__HOST=hecate-db \
    -e PG__PORT=5432 \
    -e PG__USER=postgres \
    -e PG__PASSWORD=postgres \
    -e PG__DBNAME=hecate \
    -e VOCAB_SCHEMA=vocab_27_feb_2026 \
    -e OPENAI_API_KEY="${OPENAI_API_KEY:-}" \
    -e UMLS_API_KEY="${UMLS_API_KEY:-}" \
    -e RUST_LOG=info \
    hecate-api:latest

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Hecate API container started${NC}"
else
    echo -e "${RED}✗ Failed to start Hecate API${NC}"
    exit 1
fi
echo ""

# Wait a moment for API initialization
echo "Waiting for Hecate API to initialize..."
echo "This may take a few minutes as write_pairs generates the concept index..."
sleep 5

# Show container status
echo ""
echo "======================================"
echo "Container Status"
echo "======================================"
docker ps --filter "name=hecate-db" --filter "name=qdrant" --filter "name=hecate-api" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
echo ""

echo "======================================"
echo -e "${GREEN}All containers started successfully!${NC}"
echo "======================================"
echo ""
echo "Services are available at:"
echo "  • Hecate API:     http://localhost:8080"
echo "  • API Docs:       http://localhost:8080/openapi/"
echo "  • Qdrant UI:      http://localhost:6333/dashboard"
echo "  • PostgreSQL:     localhost:5432"
echo ""
echo "Useful commands:"
echo "  • View API logs:       docker logs -f $API_CONTAINER"
echo "  • View Qdrant logs:    docker logs -f $QDRANT_CONTAINER"
echo "  • View PostgreSQL logs: docker logs -f $DB_CONTAINER"
echo "  • Stop all containers: ./stop.sh"
echo "  • Remove all data:     ./cleanup.sh"
echo ""
