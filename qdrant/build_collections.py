#!/usr/bin/env python3
"""
Build Qdrant collections for Hecate.

Creates 'meddra' (concept names) and 'synonyms' collections using
local BAAI/bge-large-en-v1.5 embeddings — no API key required.

Supports pause/resume: kill the process anytime, restart to continue
from last checkpoint. Checkpoints saved in .checkpoint_<collection>.
"""

import os
import sys
import uuid
from typing import List, Optional
from dataclasses import dataclass
from pathlib import Path

import psycopg2
from psycopg2.extras import RealDictCursor
from sentence_transformers import SentenceTransformer
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, VectorParams, PointStruct
from tqdm import tqdm

try:
    from dotenv import load_dotenv
    dotenv_path = Path(__file__).parent / '.env'
    if dotenv_path.exists():
        load_dotenv(dotenv_path)
        print(f"Loaded config from {dotenv_path}")
except ImportError:
    pass

CHECKPOINT_DIR = Path(__file__).parent


@dataclass
class ConceptData:
    concept_id: int
    concept_name: str
    domain_id: str
    vocabulary_id: str
    concept_class_id: str
    concept_code: str
    standard_concept: Optional[str]
    invalid_reason: Optional[str]


class CollectionBuilder:
    EMBEDDING_MODEL = "BAAI/bge-large-en-v1.5"
    EMBEDDING_DIMENSIONS = 1024
    BATCH_SIZE = 128

    def __init__(self, pg_host, pg_port, pg_user, pg_password, pg_dbname,
                 vocab_schema, qdrant_url):
        self.pg_config = {
            "host": pg_host,
            "port": pg_port,
            "user": pg_user,
            "password": pg_password,
            "dbname": pg_dbname,
        }
        self.vocab_schema = vocab_schema
        self.qdrant_client = QdrantClient(url=qdrant_url)

        print(f"Loading embedding model: {self.EMBEDDING_MODEL}")
        print("(First run downloads ~1.3GB to ~/.cache/huggingface/)")
        device = self._get_device()
        print(f"Using device: {device}")
        self.model = SentenceTransformer(self.EMBEDDING_MODEL, device=device)
        print(f"Model loaded (dimensions: {self.EMBEDDING_DIMENSIONS})")

    def _get_device(self) -> str:
        try:
            import torch
            if torch.backends.mps.is_available():
                return "mps"
        except Exception:
            pass
        return "cpu"

    def get_pg_connection(self):
        return psycopg2.connect(**self.pg_config, cursor_factory=RealDictCursor)

    # ── Checkpoint helpers ────────────────────────────────────────────────

    def _checkpoint_path(self, collection_name: str) -> Path:
        return CHECKPOINT_DIR / f".checkpoint_{collection_name}"

    def _load_checkpoint(self, collection_name: str) -> Optional[str]:
        path = self._checkpoint_path(collection_name)
        if path.exists():
            return path.read_text().strip() or None
        return None

    def _save_checkpoint(self, collection_name: str, last_name: str):
        self._checkpoint_path(collection_name).write_text(last_name)

    def _clear_checkpoint(self, collection_name: str):
        path = self._checkpoint_path(collection_name)
        if path.exists():
            path.unlink()

    def _collection_exists(self, name: str) -> bool:
        try:
            self.qdrant_client.get_collection(name)
            return True
        except Exception:
            return False

    def _collection_count(self, name: str) -> int:
        try:
            return self.qdrant_client.get_collection(name).points_count
        except Exception:
            return 0

    # ── Core helpers ──────────────────────────────────────────────────────

    def concept_to_payload(self, concept: ConceptData) -> dict:
        return {
            "concept_id": concept.concept_id,
            "concept_name": concept.concept_name,
            "domain_id": concept.domain_id,
            "vocabulary_id": concept.vocabulary_id,
            "concept_class_id": concept.concept_class_id,
            "concept_code": concept.concept_code,
            "standard_concept": concept.standard_concept,
            "invalid_reason": concept.invalid_reason,
        }

    def generate_embeddings(self, texts: List[str]) -> List[List[float]]:
        if not texts:
            return []
        prefixed = [f"Represent this medical term for retrieval: {t}" for t in texts]
        embeddings = self.model.encode(
            prefixed,
            batch_size=self.BATCH_SIZE,
            show_progress_bar=False,
            normalize_embeddings=True,
        )
        return embeddings.tolist()

    def _stream_name_groups(self, query: str, params: tuple = ()):
        """Server-side cursor stream — yields (name, [ConceptData]) groups."""
        conn = self.get_pg_connection()
        try:
            with conn.cursor(name="stream_cur") as cur:
                cur.itersize = 2000
                cur.execute(query, params)
                current_name = None
                group: List[ConceptData] = []
                for row in cur:
                    name = row['name']
                    concept = ConceptData(
                        concept_id=row['concept_id'],
                        concept_name=row['concept_name'],
                        domain_id=row['domain_id'],
                        vocabulary_id=row['vocabulary_id'],
                        concept_class_id=row['concept_class_id'],
                        concept_code=row['concept_code'],
                        standard_concept=row['standard_concept'],
                        invalid_reason=row['invalid_reason'],
                    )
                    if name != current_name:
                        if current_name is not None:
                            yield current_name, group
                        current_name = name
                        group = [concept]
                    else:
                        group.append(concept)
                if current_name is not None:
                    yield current_name, group
        finally:
            conn.close()

    def _build_collection_streaming(
        self,
        collection_name: str,
        query_fresh: str,
        query_resume: str,
        total_hint: int,
    ):
        checkpoint = self._load_checkpoint(collection_name)
        resuming = checkpoint is not None and self._collection_exists(collection_name)

        if resuming:
            already_done = self._collection_count(collection_name)
            print(f"Resuming '{collection_name}' from '{checkpoint}' ({already_done:,} points already uploaded)")
            query = query_resume
            params = (checkpoint,)
        else:
            if checkpoint:
                print(f"Checkpoint found but collection missing — starting fresh")
            print(f"Creating collection '{collection_name}'...")
            try:
                self.qdrant_client.delete_collection(collection_name)
                print(f"  Deleted existing '{collection_name}'")
            except Exception:
                pass
            self.qdrant_client.create_collection(
                collection_name=collection_name,
                vectors_config=VectorParams(size=self.EMBEDDING_DIMENSIONS, distance=Distance.COSINE),
            )
            print(f"  Created '{collection_name}'")
            already_done = 0
            query = query_fresh
            params = ()

        print(f"Uploading to '{collection_name}' (streaming, Ctrl+C to pause)...")

        batch_names: List[str] = []
        batch_groups: List[List[ConceptData]] = []
        uploaded = 0
        last_name = checkpoint or ""

        with tqdm(total=total_hint, initial=already_done, desc="Embed+upload", unit="names") as pbar:
            try:
                for name, group in self._stream_name_groups(query, params):
                    batch_names.append(name)
                    batch_groups.append(group)
                    last_name = name

                    if len(batch_names) >= self.BATCH_SIZE:
                        self._flush(collection_name, batch_names, batch_groups, pbar)
                        self._save_checkpoint(collection_name, last_name)
                        uploaded += len(batch_names)
                        batch_names = []
                        batch_groups = []

                # flush remainder
                if batch_names:
                    self._flush(collection_name, batch_names, batch_groups, pbar)
                    uploaded += len(batch_names)

            except KeyboardInterrupt:
                if last_name:
                    self._save_checkpoint(collection_name, last_name)
                print(f"\nPaused. Checkpoint saved: '{last_name}'")
                print(f"Restart the script to continue from this point.")
                sys.exit(0)

        self._clear_checkpoint(collection_name)
        print(f"Uploaded {uploaded:,} new points to '{collection_name}' (total: {already_done + uploaded:,})")

    def _flush(self, collection_name, batch_names, batch_groups, pbar):
        embeddings = self.generate_embeddings(batch_names)
        points = [
            PointStruct(
                id=str(uuid.uuid4()),
                vector=emb,
                payload={
                    "concept_name": n,
                    "concept_name_lower": n.lower(),
                    "concepts": [self.concept_to_payload(c) for c in grp],
                }
            )
            for n, emb, grp in zip(batch_names, embeddings, batch_groups)
        ]
        self.qdrant_client.upsert(collection_name=collection_name, points=points)
        pbar.update(len(points))

    # ── Public build methods ──────────────────────────────────────────────

    def build_meddra_collection(self):
        print("\n" + "="*60)
        print("Building 'meddra' collection")
        print("="*60)
        schema = self.vocab_schema
        fresh = f"""
            SELECT concept_name AS name, concept_id, concept_name, domain_id,
                   vocabulary_id, concept_class_id, concept_code,
                   standard_concept, invalid_reason
            FROM {schema}.concept
            ORDER BY concept_name, concept_id
        """
        resume = f"""
            SELECT concept_name AS name, concept_id, concept_name, domain_id,
                   vocabulary_id, concept_class_id, concept_code,
                   standard_concept, invalid_reason
            FROM {schema}.concept
            WHERE concept_name > %s
            ORDER BY concept_name, concept_id
        """
        self._build_collection_streaming("meddra", fresh, resume, total_hint=4_700_000)

    def build_synonyms_collection(self):
        print("\n" + "="*60)
        print("Building 'synonyms' collection")
        print("="*60)
        schema = self.vocab_schema
        fresh = f"""
            SELECT cs.concept_synonym_name AS name,
                   c.concept_id, c.concept_name, c.domain_id,
                   c.vocabulary_id, c.concept_class_id, c.concept_code,
                   c.standard_concept, c.invalid_reason
            FROM {schema}.concept_synonym cs
            JOIN {schema}.concept c ON cs.concept_id = c.concept_id
            ORDER BY cs.concept_synonym_name, c.concept_id
        """
        resume = f"""
            SELECT cs.concept_synonym_name AS name,
                   c.concept_id, c.concept_name, c.domain_id,
                   c.vocabulary_id, c.concept_class_id, c.concept_code,
                   c.standard_concept, c.invalid_reason
            FROM {schema}.concept_synonym cs
            JOIN {schema}.concept c ON cs.concept_id = c.concept_id
            WHERE cs.concept_synonym_name > %s
            ORDER BY cs.concept_synonym_name, c.concept_id
        """
        self._build_collection_streaming("synonyms", fresh, resume, total_hint=5_000_000)

    def build_all(self):
        self.build_meddra_collection()
        self.build_synonyms_collection()
        print("\n" + "="*60)
        print("All collections built successfully!")
        print("="*60)


def main():
    config = {
        "pg_host":      os.environ.get("PG_HOST", "localhost"),
        "pg_port":      int(os.environ.get("PG_PORT", 5432)),
        "pg_user":      os.environ.get("PG_USER", "postgres"),
        "pg_password":  os.environ.get("PG_PASSWORD", ""),
        "pg_dbname":    os.environ.get("PG_DBNAME", "hecate"),
        "vocab_schema": os.environ.get("VOCAB_SCHEMA", "vocab_27_feb_2026"),
        "qdrant_url":   os.environ.get("QDRANT_URL", "http://localhost:6333"),
    }

    if not config["pg_password"]:
        print("Warning: PG_PASSWORD is empty", file=sys.stderr)

    print("Configuration:")
    print(f"  PostgreSQL: {config['pg_user']}@{config['pg_host']}:{config['pg_port']}/{config['pg_dbname']}")
    print(f"  Vocab Schema: {config['vocab_schema']}")
    print(f"  Qdrant URL: {config['qdrant_url']}")
    print(f"  Embedding Model: {CollectionBuilder.EMBEDDING_MODEL} (local)")
    print()

    builder = CollectionBuilder(**config)
    builder.build_all()


if __name__ == "__main__":
    main()
