#!/usr/bin/env python3
"""ETL script to load OMOP vocabulary concepts from PostgreSQL into Apache Solr."""

import os
import sys
import json
import argparse
from datetime import date, datetime

import psycopg2
import psycopg2.extras
import requests
from tqdm import tqdm

SOLR_URL = os.environ.get("SOLR_URL", "http://localhost:8983/solr/omop_concepts")
BATCH_SIZE = 5000


def get_pg_connection():
    """Create PostgreSQL connection using the same env vars as the Rust API."""
    return psycopg2.connect(
        host=os.environ.get("PG__HOST", "127.0.0.1"),
        port=os.environ.get("PG__PORT", "5432"),
        dbname=os.environ.get("PG__DBNAME", "hecate"),
        user=os.environ.get("PG__USER", "postgres"),
        password=os.environ.get("PG__PASSWORD", "postgres"),
    )


def get_concept_count(conn, vocab_schema):
    """Get total concept count for progress bar."""
    with conn.cursor() as cur:
        cur.execute(f"SELECT COUNT(*) FROM {vocab_schema}.concept")
        return cur.fetchone()[0]


def extract_concepts(conn, vocab_schema):
    """Extract concepts using a server-side cursor for memory efficiency."""
    cursor = conn.cursor(
        name="concept_cursor", cursor_factory=psycopg2.extras.RealDictCursor
    )
    cursor.itersize = BATCH_SIZE
    query = f"""
        SELECT concept_id, concept_name, domain_id, vocabulary_id,
               concept_class_id, standard_concept, concept_code,
               invalid_reason, valid_start_date, valid_end_date
        FROM {vocab_schema}.concept
    """
    cursor.execute(query)
    return cursor


def load_record_counts():
    """Load record counts from the same JSON file the Rust API uses."""
    path = os.environ.get(
        "RECORD_COUNTS_PATH",
        os.path.join(os.path.dirname(__file__), "..", "api", "ConceptRecordCounts.json"),
    )
    path = os.path.abspath(path)
    if os.path.exists(path):
        with open(path) as f:
            data = json.load(f)
        print(f"Loaded {len(data)} record counts from {path}")
        return data
    print(f"No record counts file found at {path}, using empty counts")
    return {}


def transform_concept(row, record_counts):
    """Transform a PostgreSQL row to a Solr document."""
    doc = dict(row)

    # Convert dates to Solr ISO 8601 format
    for date_field in ("valid_start_date", "valid_end_date"):
        val = doc.get(date_field)
        if val and isinstance(val, (date, datetime)):
            doc[date_field] = val.isoformat() + "T00:00:00Z"
        elif val is None:
            del doc[date_field]

    # Replace None with empty string for string fields
    doc["standard_concept"] = doc.get("standard_concept") or ""
    doc["invalid_reason"] = doc.get("invalid_reason") or ""

    # Enrich with record count
    doc["record_count"] = record_counts.get(str(doc["concept_id"]), 0)

    return doc


def load_to_solr(docs, solr_url):
    """POST a batch of documents to Solr."""
    response = requests.post(
        f"{solr_url}/update/json/docs?commit=false",
        json=docs,
        headers={"Content-Type": "application/json"},
        timeout=120,
    )
    response.raise_for_status()


def main():
    parser = argparse.ArgumentParser(
        description="Load OMOP vocabulary concepts from PostgreSQL into Apache Solr"
    )
    parser.add_argument(
        "--clear",
        action="store_true",
        help="Delete all existing documents before loading",
    )
    parser.add_argument(
        "--no-commit",
        action="store_true",
        help="Skip final commit (useful if loading in stages)",
    )
    args = parser.parse_args()

    vocab_schema = os.environ.get("VOCAB_SCHEMA")
    if not vocab_schema:
        sys.exit("Error: VOCAB_SCHEMA environment variable must be set")

    solr_url = SOLR_URL
    print(f"Solr URL: {solr_url}")
    print(f"Vocabulary schema: {vocab_schema}")

    # Verify Solr is reachable
    try:
        resp = requests.get(f"{solr_url}/admin/ping", timeout=10)
        resp.raise_for_status()
        print("Solr ping: OK")
    except requests.RequestException as e:
        sys.exit(f"Error: Cannot reach Solr at {solr_url}: {e}")

    if args.clear:
        print("Clearing existing documents...")
        requests.post(
            f"{solr_url}/update",
            json={"delete": {"query": "*:*"}},
            headers={"Content-Type": "application/json"},
            timeout=30,
        )
        requests.get(f"{solr_url}/update?commit=true", timeout=30)
        print("Cleared.")

    record_counts = load_record_counts()

    conn = get_pg_connection()
    total_count = get_concept_count(conn, vocab_schema)
    print(f"Total concepts to load: {total_count}")

    cursor = extract_concepts(conn, vocab_schema)

    total_loaded = 0
    batch = []

    with tqdm(total=total_count, unit="concepts") as pbar:
        for row in cursor:
            batch.append(transform_concept(row, record_counts))
            if len(batch) >= BATCH_SIZE:
                load_to_solr(batch, solr_url)
                total_loaded += len(batch)
                pbar.update(len(batch))
                batch = []

        if batch:
            load_to_solr(batch, solr_url)
            total_loaded += len(batch)
            pbar.update(len(batch))

    if not args.no_commit:
        print("Committing...")
        requests.get(f"{solr_url}/update?commit=true", timeout=120)

    print(f"Done. Total concepts loaded: {total_loaded}")
    cursor.close()
    conn.close()


if __name__ == "__main__":
    main()
