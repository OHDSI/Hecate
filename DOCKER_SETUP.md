# Hecate Docker Setup Guide

This guide covers setting up and running Hecate using Docker with three separate containers managed by shell scripts.

## Overview

The Hecate deployment consists of three Docker containers:

1. **hecate-db** - PostgreSQL 15 database with OHDSI vocabulary
2. **qdrant** - Qdrant vector database with concept embeddings
3. **hecate-api** - Rust API server

All three containers run on a custom Docker network (`hecate-network`) and communicate using service names for DNS resolution.

## Prerequisites

- **Docker** installed and running (Docker Desktop on macOS/Windows, or Docker Engine on Linux)
- **OHDSI vocabulary data** obtained from [Athena](https://athena.ohdsi.org/)
- **Qdrant collections** with concept embeddings (generated with OpenAI embeddings)
- **OpenAI API key** for embedding unknown search queries
- **UMLS API key** (optional) for concept definitions - get one at [UMLS UTS](https://uts.nlm.nih.gov/uts/)

## Architecture

```
┌─────────────────────────────────────────────────┐
│         Docker Network: hecate-network          │
│                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────┐│
│  │  hecate-db  │  │   qdrant    │  │ hecate- ││
│  │             │  │             │  │   api   ││
│  │ PostgreSQL  │  │   Vector    │  │  Rust   ││
│  │     15      │  │   Database  │  │  Server ││
│  │             │  │             │  │         ││
│  │ :5432       │  │ :6333 :6334 │  │  :8080  ││
│  └─────────────┘  └─────────────┘  └─────────┘│
│         │                 │               │    │
│         └─────────────────┴───────────────┘    │
└─────────────────────────────────────────────────┘
         │                 │               │
    localhost:5432   localhost:6333  localhost:8080
```

## Setup Steps

### Step 1: Prepare PostgreSQL Data

1. Download vocabulary CSV files from Athena
2. Copy them to `postgres/data/`
3. Create a SQL file to import them (see `postgres/data/README.md` for examples)

**Required tables:**
- `concept`
- `concept_relationship`
- `concept_ancestor`
- `concept_synonym`
- `relationship`
- `phoebe` (optional, for recommendations)

### Step 2: Prepare Qdrant Collections

You need to create the `meddra` and `synonyms` collections with concept embeddings.

#### Option A: Copy Existing Qdrant Storage (Fastest)

```bash
# If you have an existing Qdrant instance with the collections already built
cp -r /path/to/your/qdrant/storage/* qdrant/collections/
```

#### Option B: Build Collections from Scratch

Build the collections from your PostgreSQL vocabulary data using the provided Python script:

```bash
cd qdrant

# Install Python dependencies
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Configure (copy and edit with your settings)
cp config.template .env

# Build collections (requires OpenAI API key - will incur costs ~$2-3)
python build_collections.py

# Copy the built collections to the docker directory
# Find your Qdrant storage directory (default: ./storage relative to Qdrant)
cp -r /path/to/qdrant/storage/* ./collections/

cd ..
```

See [qdrant/README.md](qdrant/README.md) for detailed instructions and cost estimates.

### Step 3: Set Environment Variables

Export required API keys:

```bash
export OPENAI_API_KEY="sk-your-openai-key-here"
export UMLS_API_KEY="your-umls-key-here"  # Optional
```

Or create a `.env` file in the project root:

```bash
OPENAI_API_KEY=sk-your-openai-key-here
UMLS_API_KEY=your-umls-key-here
```

Then source it:

```bash
source .env
```

### Step 4: Build Docker Images

Run the build script to create all three Docker images:

```bash
./build.sh
```

This will:
1. Build `hecate-postgres:latest` with your OHDSI vocabulary data
2. Build `hecate-qdrant:latest` with your collection data
3. Build `hecate-api:latest` with the Rust application

Expected output:
```
======================================
Building Hecate Docker Images
======================================

Building PostgreSQL image...
✓ PostgreSQL image built successfully

Building Qdrant image...
✓ Qdrant image built successfully

Building Hecate API image...
✓ Hecate API image built successfully

======================================
All images built successfully!
======================================
```

### Step 5: Start All Containers

Run the start script:

```bash
./run.sh
```

This will:
1. Create the Docker network
2. Start PostgreSQL and wait for it to be ready
3. Start Qdrant and wait for it to be ready
4. Start Hecate API (which runs `write_pairs` to generate the concept index, then starts the API server)

Expected output:
```
======================================
Starting Hecate Containers
======================================

Creating Docker network: hecate-network
✓ Network created

Starting PostgreSQL...
✓ PostgreSQL container started

Starting Qdrant...
✓ Qdrant container started

Waiting for PostgreSQL to be ready...
✓ PostgreSQL is ready

Waiting for Qdrant to be ready...
✓ Qdrant is ready

Starting Hecate API...
✓ Hecate API container started

======================================
All containers started successfully!
======================================

Services are available at:
  • Hecate API:     http://localhost:8080
  • API Docs:       http://localhost:8080/openapi/
  • Qdrant UI:      http://localhost:6333/dashboard
  • PostgreSQL:     localhost:5432
```

**Note:** The first startup may take several minutes as the `write_pairs` utility generates the concept index from Qdrant.

## Verification

### Check Container Status

```bash
docker ps --filter "name=hecate" --filter "name=qdrant"
```

All three containers should show "Up" status.

### Test API Health

```bash
curl http://localhost:8080/health
```

Should return a 200 status.

### Test Search

```bash
curl "http://localhost:8080/api/search?query=diabetes"
```

Should return concept search results.

### Check Logs

View logs for each service:

```bash
# API logs (most important)
docker logs -f hecate-api

# Qdrant logs
docker logs -f qdrant

# PostgreSQL logs
docker logs -f hecate-db
```

### Access Qdrant Dashboard

Open http://localhost:6333/dashboard in your browser to view collections and perform test queries.

## Management

### Stop Containers

```bash
./stop.sh
```

Stops and removes all three containers. Data volumes are preserved.

### Restart Containers

```bash
./run.sh
```

You can run this again after stopping to restart all services.

### View Logs

```bash
# Follow API logs
docker logs -f hecate-api

# View last 100 lines
docker logs --tail 100 hecate-api

# View all logs
docker logs hecate-api
```

### Execute Commands in Containers

```bash
# PostgreSQL
docker exec -it hecate-db psql -U postgres -d hecate

# Check concept count
docker exec -it hecate-db psql -U postgres -d hecate -c "SELECT COUNT(*) FROM vocab_27_feb_2026.concept;"

# Qdrant
docker exec -it qdrant /bin/sh

# API
docker exec -it hecate-api /bin/sh
```

### Rebuild After Changes

If you need to rebuild images after making changes:

```bash
# Stop containers
./stop.sh

# Rebuild images
./build.sh

# Start again
./run.sh
```

### Complete Cleanup

To remove everything including data volumes:

```bash
./cleanup.sh
```

**⚠️ WARNING:** This permanently deletes all data including:
- PostgreSQL database (OHDSI vocabulary)
- Qdrant vector storage (concept embeddings)
- Docker network

You'll need to rebuild and re-populate everything from scratch.

## Configuration

### Environment Variables

The `run.sh` script sets these environment variables for the API container:

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_ADDR` | `0.0.0.0:8080` | API bind address |
| `QDRANT_URI` | `http://qdrant:6334` | Qdrant gRPC endpoint |
| `VECTORDB_DATA_PATH` | `/app/all_pairs.txt` | Path to concept index file |
| `CORS_ORIGINS` | `http://localhost:5173` | Allowed CORS origins |
| `PG__HOST` | `hecate-db` | PostgreSQL host (service name) |
| `PG__PORT` | `5432` | PostgreSQL port |
| `PG__USER` | `postgres` | PostgreSQL username |
| `PG__PASSWORD` | `postgres` | PostgreSQL password |
| `PG__DBNAME` | `hecate` | Database name |
| `VOCAB_SCHEMA` | `vocab_27_feb_2026` | OHDSI vocabulary schema name |
| `OPENAI_API_KEY` | (from host) | OpenAI API key |
| `UMLS_API_KEY` | (from host) | UMLS API key |
| `RUST_LOG` | `info` | Logging level |

To customize these values, edit `run.sh` before running it.

### Ports

| Service | Internal Port | External Port | Description |
|---------|---------------|---------------|-------------|
| hecate-api | 8080 | 8080 | API HTTP server |
| qdrant | 6333 | 6333 | Qdrant HTTP API |
| qdrant | 6334 | 6334 | Qdrant gRPC API |
| hecate-db | 5432 | 5432 | PostgreSQL |

To change external ports, edit the `-p` flags in `run.sh`.

### Volumes

Two named Docker volumes are created for data persistence:

- `hecate-postgres-data` - PostgreSQL database files
- `hecate-qdrant-storage` - Qdrant collection data

To inspect volumes:

```bash
docker volume ls | grep hecate
docker volume inspect hecate-postgres-data
```

## Troubleshooting

### Problem: "PostgreSQL failed to start"

**Solution:**
1. Check logs: `docker logs hecate-db`
2. Ensure `postgres/data/` contains valid SQL dumps or CSV files
3. Check SQL syntax errors in initialization scripts
4. Verify file permissions

### Problem: "Qdrant failed to start"

**Solution:**
1. Check logs: `docker logs qdrant`
2. Verify `qdrant/collections/` directory structure
3. Check for Qdrant version compatibility issues
4. Try starting with empty collections first

### Problem: "write_pairs taking too long"

**Context:** The `write_pairs` utility scrolls through all Qdrant points to build the concept index. For large vocabularies (millions of concepts), this can take 5-10 minutes.

**Solution:**
1. Be patient - this is normal for first-time setup
2. Monitor progress: `docker logs -f hecate-api`
3. Ensure Qdrant collections are properly loaded
4. If it hangs for >15 minutes, restart: `./stop.sh && ./run.sh`

### Problem: "API can't connect to PostgreSQL"

**Solution:**
1. Verify PostgreSQL is running: `docker ps | grep hecate-db`
2. Check network connectivity: `docker exec hecate-api ping hecate-db`
3. Verify environment variables: `docker inspect hecate-api | grep PG__`
4. Check PostgreSQL accepts connections: `docker exec hecate-db pg_isready`

### Problem: "API can't connect to Qdrant"

**Solution:**
1. Verify Qdrant is running: `docker ps | grep qdrant`
2. Check network connectivity: `docker exec hecate-api ping qdrant`
3. Test Qdrant API: `curl http://localhost:6333/collections`
4. Verify collections exist: `curl http://localhost:6333/collections`

### Problem: "No search results returned"

**Possible causes:**
1. Collections are empty - check Qdrant dashboard
2. PostgreSQL vocabulary tables not loaded - check with `psql`
3. `write_pairs` didn't complete - check API logs
4. Wrong `VOCAB_SCHEMA` - verify schema name matches your database

### Problem: "Permission denied on .sh scripts"

**Solution:**
```bash
chmod +x build.sh run.sh stop.sh cleanup.sh postgres/init-db.sh
```

### Problem: "Network already exists" error

**Solution:**
The script handles this automatically. If you see issues:
```bash
docker network rm hecate-network
./run.sh
```

## Development Workflow

### Making Changes to the API

1. Edit code in `api/src/`
2. Rebuild the API image:
   ```bash
   docker build -t hecate-api:latest .
   ```
3. Restart the API container:
   ```bash
   docker stop hecate-api
   docker rm hecate-api
   ./run.sh  # This will skip postgres/qdrant since they're already running
   ```

### Updating PostgreSQL Data

1. Stop all containers: `./stop.sh`
2. Update files in `postgres/data/`
3. Rebuild postgres image: `docker build -t hecate-postgres:latest ./postgres`
4. Remove the old volume: `docker volume rm hecate-postgres-data`
5. Start again: `./run.sh`

### Updating Qdrant Collections

1. Stop all containers: `./stop.sh`
2. Update collections in `qdrant/collections/`
3. Rebuild qdrant image: `docker build -t hecate-qdrant:latest ./qdrant`
4. Remove the old volume: `docker volume rm hecate-qdrant-storage`
5. Start again: `./run.sh`

## Comparison with Docker Compose

This setup replaces the original `docker-compose.yml` with shell scripts for users who don't have Docker Compose installed.

**Advantages:**
- No Docker Compose dependency
- Easier to understand and customize
- Better health checking and startup sequencing
- Explicit error handling

**Disadvantages:**
- More verbose than `docker-compose.yml`
- Manual network management
- No automatic restart policies (can be added with `--restart unless-stopped`)

If you prefer Docker Compose, the original `docker-compose.yml` is still available in the repository.

## Additional Resources

- [API Documentation](./api/README.md) - Hecate API details
- [Local Installation Guide](./api/LOCAL_INSTALL.md) - Non-Docker setup instructions
- [UI Documentation](./ui/README.md) - Frontend setup
- [MCP Server](./mcp/README.md) - Model Context Protocol integration
- [Qdrant Documentation](https://qdrant.tech/documentation/) - Vector database docs
- [OHDSI Vocabulary](https://www.ohdsi.org/analytic-tools/athena-standardized-vocabularies/) - Vocabulary information

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review container logs
3. Consult the detailed READMEs in `postgres/data/` and `qdrant/collections/`
4. Open an issue on the project repository

---

**Note:** This setup is designed for local development and testing. For production deployments, consider:
- Using Docker Compose or Kubernetes for orchestration
- Implementing proper secrets management
- Setting up SSL/TLS certificates
- Configuring backup strategies
- Implementing monitoring and logging solutions
