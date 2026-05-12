# Qdrant Collections Directory

This directory should contain your Qdrant collections with concept embeddings for the Hecate semantic search engine.

## Required Collections

The Hecate API requires two Qdrant collections:

### 1. `meddra` - Main Concept Collection

Each point contains:
- **id**: UUID
- **vector**: 1024-dimensional embedding (generated with OpenAI `text-embedding-3-large`, dimensions=1024)
- **payload**:
  - `concept_name` (string)
  - `concept_name_lower` (string) - lowercased, used for exact matching
  - `concepts` (array of objects) - each with: `concept_id`, `concept_name`, `domain_id`, `vocabulary_id`, `concept_class_id`, `concept_code`, `standard_concept`, `invalid_reason`

### 2. `synonyms` - Synonym Collection

Same structure as `meddra`, but built from concept synonyms rather than primary concept names.

## How to Add Your Collections

### Option 1: Copy Existing Qdrant Storage (Recommended)

If you already have a running Qdrant instance with the collections:

1. Stop your Qdrant instance
2. Copy the entire storage directory to this location:
   ```bash
   cp -r /path/to/your/qdrant/storage/* /path/to/Hecate/qdrant/collections/
   ```
3. The structure should look like:
   ```
   qdrant/collections/
   ├── collection/
   │   ├── meddra/
   │   └── synonyms/
   └── meta.json
   ```

### Option 2: Load Qdrant Snapshots

If you have Qdrant collection snapshots:

1. Place snapshot files in this directory:
   ```
   qdrant/collections/
   ├── meddra_snapshot.snapshot
   └── synonyms_snapshot.snapshot
   ```
2. Create a startup script to restore them (requires modification of the Dockerfile)

### Option 3: Start with Empty Collections

If you need to generate embeddings from scratch:

1. Leave this directory empty
2. Build and run the containers
3. Use the Qdrant API to create collections:
   ```bash
   curl -X PUT 'http://localhost:6333/collections/meddra' \
     -H 'Content-Type: application/json' \
     -d '{
       "vectors": {
         "size": 1024,
         "distance": "Cosine"
       }
     }'
   ```
4. Generate and upload embeddings (see `api/LOCAL_INSTALL.md` for details)

> **Note:** Generating embeddings for the full OHDSI vocabulary requires significant OpenAI API usage (potentially $100-500 depending on vocabulary size).

## Verification

After building and running the container, verify collections exist:

```bash
# Check collections via HTTP API
curl http://localhost:6333/collections

# Or use Qdrant's web UI
open http://localhost:6333/dashboard
```

You should see both `meddra` and `synonyms` collections listed.

## Collection Structure Example

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "vector": [0.123, -0.456, 0.789, ...],  // 1024 dimensions
  "payload": {
    "concept_name": "Type 2 diabetes mellitus",
    "concept_name_lower": "type 2 diabetes mellitus",
    "concepts": [
      {
        "concept_id": 201826,
        "concept_name": "Type 2 diabetes mellitus",
        "domain_id": "Condition",
        "vocabulary_id": "SNOMED",
        "concept_class_id": "Clinical Finding",
        "concept_code": "44054006",
        "standard_concept": "S",
        "invalid_reason": null
      }
    ]
  }
}
```

## Troubleshooting

If collections don't load:
- Check container logs: `docker logs qdrant`
- Verify file permissions (files must be readable)
- Ensure directory structure matches Qdrant's storage format
- Check Qdrant version compatibility (collections from much older versions may not be compatible)

For more information, see:
- `api/LOCAL_INSTALL.md` - Detailed setup guide with embedding generation
- `DOCKER_SETUP.md` - Docker deployment instructions
- [Qdrant Documentation](https://qdrant.tech/documentation/) - Official Qdrant docs
