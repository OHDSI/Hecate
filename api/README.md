# Hecate API

The main API backend for Hecate, providing concept search and data retrieval endpoints.

## Features

- **Concept Search** - Semantic search using vector embeddings
- **Concept Details** - Detailed concept information and metadata
- **Related Concepts** - Hierarchical and relationship-based concept discovery
- **Database Integration** - PostgreSQL for vocabulary data, Qdrant for vector search

## Quick Start

1. Copy the environment configuration:
   ```bash
   cp .env.example .env
   ```

2. Update the `.env` file with your specific configuration values

3. Run the application:
   ```bash
   cargo run
   ```

## Prerequisites

- Rust
- PostgreSQL database with OHDSI vocabulary
- Qdrant vector database with concept embeddings

## Configuration

The API uses environment variables for configuration. Copy `.env.example` to `.env` and update the following settings:

- `SERVER_ADDR` - Server address and port (default: 127.0.0.1:8080)
- `QDRANT_URI` - Qdrant vector database URL (default: http://localhost:6334)
- `VECTORDB_DATA_PATH` - Path to vector data file (default: sample_data.txt)
- `CORS_ORIGINS` - Allowed CORS origins (default: http://localhost:5173)
- `PG__USER` - PostgreSQL username
- `PG__PASSWORD` - PostgreSQL password
- `PG__HOST` - PostgreSQL host (default: 127.0.0.1)
- `PG__PORT` - PostgreSQL port (default: 5432)
- `PG__DBNAME` - PostgreSQL database name
- `PG__POOL_MAX_SIZE` - Database connection pool size (default: 16)
- `UMLS_API_KEY` - UMLS API key for concept definitions

## API Endpoints

The service provides REST endpoints for:
- Concept search and filtering
- Concept details and metadata
- Related concept discovery
- Hierarchical concept browsing

## Database Schema

The API expects standard OHDSI vocabulary tables including:
- `concept` - Core concept definitions
- `concept_relationship` - Concept relationships
- `concept_ancestor` - Hierarchical relationships
