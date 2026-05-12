#!/bin/bash
set -e

echo "======================================"
echo "Building Hecate Docker Images"
echo "======================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if postgres data directory exists and warn if empty
if [ ! -d "postgres/data" ]; then
    echo -e "${RED}ERROR: postgres/data directory not found${NC}"
    exit 1
fi

if [ -z "$(ls -A postgres/data/*.sql 2>/dev/null)" ] && [ -z "$(ls -A postgres/data/*.csv 2>/dev/null)" ]; then
    echo -e "${YELLOW}WARNING: postgres/data/ appears to be empty${NC}"
    echo -e "${YELLOW}You may need to add OHDSI vocabulary data before running.${NC}"
    echo -e "${YELLOW}See postgres/data/README.md for instructions.${NC}"
    echo ""
fi

# Check if qdrant collections directory exists
if [ ! -d "qdrant/collections" ]; then
    echo -e "${RED}ERROR: qdrant/collections directory not found${NC}"
    exit 1
fi

if [ -z "$(ls -A qdrant/collections/ 2>/dev/null | grep -v README.md | grep -v .gitkeep)" ]; then
    echo -e "${YELLOW}WARNING: qdrant/collections/ appears to be empty${NC}"
    echo -e "${YELLOW}You may need to add Qdrant collection data before running.${NC}"
    echo -e "${YELLOW}See qdrant/collections/README.md for instructions.${NC}"
    echo ""
fi

# Build PostgreSQL image
echo "Building PostgreSQL image (this may take 10-20 minutes as it loads CSV data)..."
docker build -t hecate-postgres:latest ./postgres
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ PostgreSQL image built successfully (database pre-initialized)${NC}"
else
    echo -e "${RED}✗ Failed to build PostgreSQL image${NC}"
    exit 1
fi
echo ""

# Build Qdrant image
echo "Building Qdrant image..."
docker build -t hecate-qdrant:latest ./qdrant
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Qdrant image built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build Qdrant image${NC}"
    exit 1
fi
echo ""

# Build Hecate API image
echo "Building Hecate API image..."
docker build -t hecate-api:latest .
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Hecate API image built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build Hecate API image${NC}"
    exit 1
fi
echo ""

echo "======================================"
echo -e "${GREEN}All images built successfully!${NC}"
echo "======================================"
echo ""
echo "Next steps:"
echo "  1. Set required environment variables (OPENAI_API_KEY, UMLS_API_KEY)"
echo "  2. Run './run.sh' to start all containers"
echo ""
