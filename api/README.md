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

2. The API uses environment variables for configuration. Copy `.env.example` to `.env` and update the file with your
   specific configuration values

3. Run the application:
   ```bash
   cargo run
   ```

## Prerequisites

- Rust
- PostgreSQL database with OHDSI vocabulary
- Qdrant vector database with concept embeddings

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
