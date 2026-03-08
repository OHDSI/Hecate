# Solr Lexical Search Setup

Hecate supports an optional Apache Solr-based lexical search alongside the primary semantic (vector) search. Lexical search is better for exact concept codes (ICD-10, SNOMED codes), known concept names, and keyword-based queries.

**This component is optional.** The API starts and functions normally without Solr. The `/api/search_lexical` endpoint returns `503 Service Unavailable` when Solr is not configured.

## Prerequisites

- Docker and Docker Compose
- Python 3.8+ (for the ETL script)
- A running PostgreSQL instance with the OMOP vocabulary loaded

## Quick Start

### 1. Start Solr

From the repository root:

```bash
docker compose up -d
```

This starts Solr 9.8 on port 8983 and creates the `omop_concepts` core with the pre-configured schema.

Verify Solr is running:

```bash
curl http://localhost:8983/solr/omop_concepts/admin/ping
```

### 2. Load Vocabulary into Solr

Install Python dependencies:

```bash
cd etl
pip install -r requirements.txt
```

Run the ETL script. The script reads from the same PostgreSQL database used by the API, using the same environment variables:

```bash
export PG__HOST=127.0.0.1
export PG__PORT=5432
export PG__DBNAME=hecate
export PG__USER=postgres
export PG__PASSWORD=postgres
export VOCAB_SCHEMA=vocab_27_feb_2026

python solr_load.py --clear
```

This streams all concepts from PostgreSQL into Solr in batches of 5000. For ~7M concepts, expect a few minutes. Progress is displayed via a progress bar.

Options:
- `--clear` removes all existing documents before loading
- `--no-commit` skips the final commit (useful for staged loading)

The script also enriches concepts with record counts from `api/ConceptRecordCounts.json` if available.

### 3. Configure the API

Add the Solr URL to your `.env` file:

```bash
SOLR_URL=http://localhost:8983/solr/omop_concepts
```

If `SOLR_URL` is not set, the API starts normally with lexical search disabled.

### 4. Test

```bash
# Lexical search
curl "http://localhost:8080/api/search_lexical?q=diabetes&limit=5"

# Search by exact concept code
curl "http://localhost:8080/api/search_lexical?q=E11.9"

# With filters
curl "http://localhost:8080/api/search_lexical?q=hypertension&vocabulary_id=SNOMED&domain_id=Condition"
```

## Solr Schema

The schema is defined in `solr/configsets/omop_concepts/conf/managed-schema.xml` and includes:

| Field | Type | Purpose |
|-------|------|---------|
| `concept_id` | pint (uniqueKey) | OMOP concept ID |
| `concept_name` | text_medical | Full-text search with medical text analysis |
| `concept_name_edge` | text_edge_ngram | Prefix/autocomplete matching (copy field) |
| `concept_name_exact` | string | Exact match boosting (copy field) |
| `concept_code` | string | Vocabulary-specific code (e.g., ICD-10 code) |
| `domain_id` | string | Clinical domain filter |
| `vocabulary_id` | string | Vocabulary source filter |
| `concept_class_id` | string | Concept classification filter |
| `standard_concept` | string | Standardization status filter |
| `record_count` | plong | Usage frequency in the OHDSI network |

### Text Analysis

The `text_medical` field type uses:
- Whitespace tokenizer
- Word delimiter graph filter (splits `acetyl-salicylic` into tokens, preserves original)
- ASCII folding (handles accented characters in European drug names)
- English minimal stemmer (light stemming appropriate for medical terms)
- Medical synonym expansion at query time (e.g., MI -> myocardial infarction)

### Query Configuration

Uses the eDisMax query parser with field boosting:
- `concept_name_exact^10` - exact name match highest priority
- `concept_code^8` - code matches high priority
- `concept_name^5` - analyzed text match
- `concept_name_edge^1` - prefix matches lowest priority
- Phrase boosting: `concept_name^10`
- Minimum match: 75% of query terms

## Medical Synonyms

Common medical abbreviations are expanded at query time via `solr/configsets/omop_concepts/conf/medical_synonyms.txt`. Add additional synonyms as needed in the format:

```
ABBREVIATION, full term
```

After modifying synonyms, reload the Solr core:

```bash
curl "http://localhost:8983/solr/admin/cores?action=RELOAD&core=omop_concepts"
```

## Architecture

```
PostgreSQL (OMOP vocab) ──ETL──> Apache Solr (lexical index)
                                        │
                                        ▼
Hecate API ──/api/search_lexical──> Solr REST API
         └──/api/search──────────> Qdrant (semantic)
```

The lexical and semantic search endpoints are independent. Clients can query either or both and merge results as needed.
