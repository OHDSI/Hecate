#!/bin/bash

echo "======================================"
echo "Cleaning Up Hecate Docker Resources"
echo "======================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Network and volume names
NETWORK_NAME="hecate-network"
POSTGRES_VOLUME="hecate-postgres-data"
QDRANT_VOLUME="hecate-qdrant-storage"

# Container names
DB_CONTAINER="hecate-db"
QDRANT_CONTAINER="qdrant"
API_CONTAINER="hecate-api"

echo -e "${RED}WARNING: This will permanently delete all data!${NC}"
echo "This includes:"
echo "  • PostgreSQL database (OHDSI vocabulary data)"
echo "  • Qdrant vector storage (concept embeddings)"
echo "  • Docker network"
echo ""
read -p "Are you sure you want to continue? (yes/no): " -r
echo ""

if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    echo "Cleanup cancelled."
    exit 0
fi

# Stop and remove containers first
echo "Stopping and removing containers..."
docker stop $API_CONTAINER $QDRANT_CONTAINER $DB_CONTAINER 2>/dev/null || true
docker rm $API_CONTAINER $QDRANT_CONTAINER $DB_CONTAINER 2>/dev/null || true
echo -e "${GREEN}✓ Containers removed${NC}"
echo ""

# Remove Docker volumes
echo "Removing Docker volumes..."
docker volume rm $POSTGRES_VOLUME 2>/dev/null && echo -e "${GREEN}✓ PostgreSQL volume removed${NC}" || echo -e "${YELLOW}PostgreSQL volume not found${NC}"
docker volume rm $QDRANT_VOLUME 2>/dev/null && echo -e "${GREEN}✓ Qdrant volume removed${NC}" || echo -e "${YELLOW}Qdrant volume not found${NC}"
echo ""

# Remove Docker network
echo "Removing Docker network..."
docker network rm $NETWORK_NAME 2>/dev/null && echo -e "${GREEN}✓ Network removed${NC}" || echo -e "${YELLOW}Network not found${NC}"
echo ""

echo "======================================"
echo -e "${GREEN}Cleanup complete!${NC}"
echo "======================================"
echo ""
echo "All Hecate Docker resources have been removed."
echo "To start fresh:"
echo "  1. Run './build.sh' to rebuild images"
echo "  2. Run './run.sh' to start containers"
echo ""
