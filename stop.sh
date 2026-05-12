#!/bin/bash

echo "======================================"
echo "Stopping Hecate Containers"
echo "======================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Container names
DB_CONTAINER="hecate-db"
QDRANT_CONTAINER="qdrant"
API_CONTAINER="hecate-api"

# Stop containers
echo "Stopping containers..."
docker stop $API_CONTAINER $QDRANT_CONTAINER $DB_CONTAINER 2>/dev/null || true

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Containers stopped${NC}"
else
    echo -e "${YELLOW}Some containers were not running${NC}"
fi

# Remove containers
echo "Removing containers..."
docker rm $API_CONTAINER $QDRANT_CONTAINER $DB_CONTAINER 2>/dev/null || true

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Containers removed${NC}"
else
    echo -e "${YELLOW}Some containers were already removed${NC}"
fi

echo ""
echo "======================================"
echo -e "${GREEN}Containers stopped and removed${NC}"
echo "======================================"
echo ""
echo "Note: Data volumes are preserved."
echo "To remove data volumes, run: ./cleanup.sh"
echo "To start again, run: ./run.sh"
echo ""
