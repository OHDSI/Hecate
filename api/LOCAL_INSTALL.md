# Hecate API - Local Installation Guide

This guide covers setting up the **hecate-api** component locally. Other components (UI, autocomplete, MCP) will be
covered separately.

## Prerequisites

- **Rust**
- **PostgreSQL**
- **Qdrant** vector database
- **OpenAI API key** for generating embeddings for unknown search queries

## 1. PostgreSQL Setup

### Load the OHDSI vocabulary

The API expects the OHDSI vocabulary tables under a dedicated schema. The schema name is configured via the
`VOCAB_SCHEMA` environment variable in your `.env` file (e.g., `VOCAB_SCHEMA=vocab_27_feb_2026`).

You need to obtain the OHDSI vocabulary CSV files from [Athena](https://athena.ohdsi.org/). Download the vocabulary
bundle and load the following tables:

```sql
CREATE SCHEMA vocab_27_feb_2026;  -- must match your VOCAB_SCHEMA env var

-- The core tables needed:
-- concept, concept_relationship, concept_ancestor, concept_synonym, relationship
```

### Load Phoebe (optional)

Phoebe provides recommended concept relationships. To enable the `/api/concepts/{id}/phoebe` endpoint, you need to load
the Phoebe data into the same schema:

```sql
CREATE TABLE vocab_27_feb_2026.phoebe  -- must match your VOCAB_SCHEMA env var
(
    concept_id_1    INTEGER,
    concept_id_2    INTEGER,
    relationship_id VARCHAR
);
```

If you don't load Phoebe, the API will still work but the Phoebe endpoint will return errors.

## 2. Qdrant Setup

### Install and start Qdrant

```bash
# Docker (easiest)
docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant
```

The API connects to Qdrant on `http://localhost:6334` (gRPC port) by default.

### Create collections and load embeddings

The API uses two Qdrant collections:

#### `meddra` — main concept collection

Each point contains:

- **id**: UUID
- **vector**: 1024-dimensional embedding (generated with OpenAI `text-embedding-3-large`, dimensions=1024)
- **payload**:
    - `concept_name` (string)
    - `concept_name_lower` (string) — lowercased, used for exact matching
    - `concepts` (array of objects) — each with: `concept_id`, `concept_name`, `domain_id`, `vocabulary_id`,
      `concept_class_id`, `concept_code`, `standard_concept`, `invalid_reason`

#### `synonyms` — synonym collection

Same structure as `meddra`, but built from concept synonyms rather than primary concept names.

> **Note:** Generating embeddings for the full vocabulary requires significant OpenAI API usage. For a minimal local
> setup you can load a subset.

## 3. Startup Data Files

The API loads two JSON files into memory at startup for performance:

### Concept index file (`vectordb_data_path`)

A txt file mapping lowercased concept names to their Qdrant point UUIDs (`HashMap<String, Vec<UUID>>`).

This can be generated from Qdrant using the `write_pairs` binary, which scrolls through all points in the
collection and builds the mapping:

```bash
cd api
cargo run --bin write_pairs

# Or specify a custom output path
cargo run --bin write_pairs -- my_output.txt
```

Output defaults to `all_pairs.txt` in the current directory. Point `VECTORDB_DATA_PATH` in your `.env` to this file.

### Concept record counts file (`ConceptRecordCounts.json`)

A JSON file mapping concept IDs to their record counts from a CDM database (`HashMap<i32, i64>`). Must be placed in the
working directory where the API is run from.

If you don't have record count data, create an empty JSON object file:

```bash
echo '{}' > ConceptRecordCounts.json
```

## 4. Environment Configuration

Create a `.env` file in the `api/` directory (or set environment variables):

```env
SERVER_ADDR=127.0.0.1:8080
QDRANT_URI=http://localhost:6334
VECTORDB_DATA_PATH=path/to/concept_index.txt
CORS_ORIGINS=http://localhost:5173

# PostgreSQL connection (deadpool-postgres config)
PG__HOST=localhost
PG__PORT=5432
PG__USER=your_pg_user
PG__PASSWORD=your_pg_password
PG__DBNAME=hecate

# Vocabulary schema name (updated ~twice a year with new OHDSI releases)
VOCAB_SCHEMA=vocab_27_feb_2026

# Cache settings (optional, these are the defaults)
CACHE_MAX_CAPACITY=64
CACHE_TTL_DAYS=21
SEARCH_CACHE_MAX_BYTES=104857600
SEARCH_CACHE_TTL_DAYS=30

# OpenAI (for embedding unknown search queries)
OPENAI_API_KEY=sk-your-key-here

# UMLS API key (optional — needed for the /api/concepts/{id}/definition endpoint)
# Get one at https://uts.nlm.nih.gov/uts/
UMLS_API_KEY=your-umls-key-here
```

> The `PG__` prefix with double underscores is the nested config format used by `confik`/`deadpool-postgres`.

## 5. Run the API

```bash
cd api
cargo run
```
