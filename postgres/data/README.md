# PostgreSQL Data Directory

This directory should contain your OHDSI vocabulary data for the Hecate database.

## Required Data

The Hecate API requires the OHDSI vocabulary tables loaded into PostgreSQL. You need to obtain these from [Athena](https://athena.ohdsi.org/).

### Required Tables

The following tables must be loaded into the schema specified by `VOCAB_SCHEMA` (default: `vocab_27_feb_2026`):

- `concept`
- `concept_relationship`
- `concept_ancestor`
- `concept_synonym`
- `relationship`

### Optional Tables

- `phoebe` - Enables the `/api/concepts/{id}/phoebe` endpoint for concept recommendations

## How to Add Your Data

### Option 2: CSV Files

1. Download CSV files from Athena
2. Place the CSV files in this directory
3. You'll need to manually create COPY commands in a SQL file to import them

Example SQL file (`load_csvs.sql`):
```sql
\set vocab_schema 'vocab_27_feb_2026'

COPY :vocab_schema.concept FROM '/docker-entrypoint-initdb.d/data/CONCEPT.csv' DELIMITER E'\t' CSV HEADER QUOTE E'\b';
COPY :vocab_schema.concept_relationship FROM '/docker-entrypoint-initdb.d/data/CONCEPT_RELATIONSHIP.csv' DELIMITER E'\t' CSV HEADER QUOTE E'\b';
-- Add more COPY commands for other tables...
```

### Option 3: Mount Existing Data Volume

If you already have a PostgreSQL data directory, you can mount it directly when running the container instead of using this build-time approach. See `DOCKER_SETUP.md` for details.

## Verification

After building and running the container, verify the data loaded correctly:

```bash
docker exec -it hecate-db psql -U postgres -d hecate -c "SELECT COUNT(*) FROM vocab_27_feb_2026.concept;"
```

You should see the total number of concepts in your vocabulary.

## Troubleshooting

If initialization fails:
- Check container logs: `docker logs hecate-db`
- Verify SQL syntax in your dump files
- Ensure the `VOCAB_SCHEMA` environment variable matches your schema name
- Check file permissions (files must be readable)

For more information, see:
- `api/LOCAL_INSTALL.md` - Detailed setup guide
- `DOCKER_SETUP.md` - Docker deployment instructions
