#!/usr/bin/env python3
"""
Build Qdrant collections for Hecate.

This script creates the 'meddra' and 'synonyms' collections by:
1. Reading concept data from PostgreSQL
2. Generating embeddings using BAAI/bge-large-en-v1.5 model
3. Creating and populating Qdrant collections

The collections can then be copied to the docker image for deployment.
"""

import os
import sys
import json
import uuid
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
import asyncio
from collections import defaultdict
from pathlib import Path

import psycopg2
from psycopg2.extras import RealDictCursor
from sentence_transformers import SentenceTransformer
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, VectorParams, PointStruct
from tqdm import tqdm

# Try to load .env file if python-dotenv is available
try:
    from dotenv import load_dotenv
    dotenv_path = Path(__file__).parent / '.env'
    if dotenv_path.exists():
        load_dotenv(dotenv_path)
        print(f"Loaded configuration from {dotenv_path}")
except ImportError:
    pass


@dataclass
class ConceptData:
    """Represents a concept from the OHDSI vocabulary."""
    concept_id: int
    concept_name: str
    domain_id: str
    vocabulary_id: str
    concept_class_id: str
    concept_code: str
    standard_concept: Optional[str]
    invalid_reason: Optional[str]


class CollectionBuilder:
    """Builds Qdrant collections from OHDSI vocabulary data."""
    
    EMBEDDING_MODEL = "BAAI/bge-large-en-v1.5"
    EMBEDDING_DIMENSIONS = 1024
    BATCH_SIZE = 128  # Optimized for M2 - process more at once
    QDRANT_BATCH_SIZE = 100  # Number of points to upload at once
    
    def __init__(
        self,
        pg_host: str,
        pg_port: int,
        pg_user: str,
        pg_password: str,
        pg_dbname: str,
        vocab_schema: str,
        qdrant_url: str,
    ):
        self.pg_config = {
            "host": pg_host,
            "port": pg_port,
            "user": pg_user,
            "password": pg_password,
            "dbname": pg_dbname,
        }
        self.vocab_schema = vocab_schema
        self.qdrant_client = QdrantClient(url=qdrant_url)
        
        # Initialize sentence transformer model
        print(f"Loading embedding model: {self.EMBEDDING_MODEL}")
        print("(First run will download ~1.3GB model - this may take a few minutes)")
        
        # Use MPS (Metal Performance Shaders) for M2 acceleration if available
        device = self._get_device()
        print(f"Using device: {device}")
        
        self.model = SentenceTransformer(
            self.EMBEDDING_MODEL,
            device=device
        )
        print(f"Model loaded successfully (dimensions: {self.EMBEDDING_DIMENSIONS})")
    
    def _get_device(self) -> str:
        """Determine the best device for M2 Mac."""
        try:
            import torch
            if torch.backends.mps.is_available():
                return "mps"  # Metal Performance Shaders for M2
        except:
            pass
        return "cpu"
        
    def get_pg_connection(self):
        """Create a PostgreSQL connection."""
        return psycopg2.connect(**self.pg_config, cursor_factory=RealDictCursor)
    
    def fetch_concepts(self) -> List[ConceptData]:
        """Fetch all concepts from PostgreSQL."""
        print("Fetching concepts from PostgreSQL...")
        conn = self.get_pg_connection()
        try:
            with conn.cursor() as cur:
                cur.execute(f"""
                    SELECT 
                        concept_id,
                        concept_name,
                        domain_id,
                        vocabulary_id,
                        concept_class_id,
                        concept_code,
                        standard_concept,
                        invalid_reason
                    FROM {self.vocab_schema}.concept
                    ORDER BY concept_id
                """)
                rows = cur.fetchall()
                concepts = [ConceptData(**row) for row in rows]
                print(f"Fetched {len(concepts)} concepts")
                return concepts
        finally:
            conn.close()
    
    def fetch_synonyms(self) -> List[tuple[str, ConceptData]]:
        """Fetch all concept synonyms with their associated concept data."""
        print("Fetching synonyms from PostgreSQL...")
        conn = self.get_pg_connection()
        try:
            with conn.cursor() as cur:
                cur.execute(f"""
                    SELECT 
                        cs.concept_synonym_name,
                        c.concept_id,
                        c.concept_name,
                        c.domain_id,
                        c.vocabulary_id,
                        c.concept_class_id,
                        c.concept_code,
                        c.standard_concept,
                        c.invalid_reason
                    FROM {self.vocab_schema}.concept_synonym cs
                    JOIN {self.vocab_schema}.concept c ON cs.concept_id = c.concept_id
                    ORDER BY cs.concept_synonym_name, c.concept_id
                """)
                rows = cur.fetchall()
                synonyms = [
                    (row['concept_synonym_name'], ConceptData(
                        concept_id=row['concept_id'],
                        concept_name=row['concept_name'],
                        domain_id=row['domain_id'],
                        vocabulary_id=row['vocabulary_id'],
                        concept_class_id=row['concept_class_id'],
                        concept_code=row['concept_code'],sentence-transformers."""
        if not texts:
            return []
        
        # BGE models benefit from adding instruction prefix for retrieval tasks
        # For medical terms, we use a simple instruction
        prefixed_texts = [f"Represent this medical term for retrieval: {text}" for text in texts]
        
        # Generate embeddings with normalization
        embeddings = self.model.encode(
            prefixed_texts,
            batch_size=self.BATCH_SIZE,
            show_progress_bar=False,
            normalize_embeddings=True  # Important for cosine similarity
        )
        
        return embeddings.tolist()
    
    def group_concepts_by_name(
        self, 
        data: List[tuple[str, ConceptData]]
    ) -> Dict[str, List[ConceptData]]:
        """Group concepts by their name (case-insensitive)."""
        grouped = defaultdict(list)
        for name, concept in data:
            grouped[name.lower()].append(concept)
        return dict(grouped)
    
    def generate_embeddings(self, texts: List[str]) -> List[List[float]]:
        """Generate embeddings for a batch of texts using OpenAI."""
        if not texts:
            return []
        
        response = self.openai_client.embeddings.create(
            input=texts,
            model=self.EMBEDDING_MODEL,
            dimensions=self.EMBEDDING_DIMENSIONS
        )
        return [item.embedding for item in response.data]
    
    def create_collection(self, collection_name: str):
        """Create a Qdrant collection with the appropriate configuration."""
        print(f"Creating collection '{collection_name}'...")
        
        # Delete if exists
        try:
            self.qdrant_client.delete_collection(collection_name)
            print(f"Deleted existing collection '{collection_name}'")
        except Exception:
            pass
        
        # Create new collection
        self.qdrant_client.create_collection(
            collection_name=collection_name,
            vectors_config=VectorParams(
                size=self.EMBEDDING_DIMENSIONS,
                distance=Distance.COSINE
            )
        )
        print(f"Created collection '{collection_name}'")
    
    def concept_to_payload(self, concept: ConceptData) -> Dict[str, Any]:
        """Convert a ConceptData object to a Qdrant payload dictionary."""
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
    concepts[0].concept_
    def upload_points(
        self,
        collection_name: str,
        name_concept_map: Dict[str, List[ConceptData]]
    ):
        """Generate embeddings and upload points to Qdrant."""
        print(f"Uploading points to '{collection_name}'...")
        
        # Prepare all names for embedding
        names = list(name_concept_map.keys())
        total_names = len(names)
        
        # Process in batches
        all_points = []
        for i in tqdm(range(0, total_names, self.BATCH_SIZE), desc="Generating embeddings"):
            batch_names = names[i:i + self.BATCH_SIZE]
            
            # Generate embeddings for this batch
            embeddings = self.generate_embeddings(batch_names)
            
            # Create points
            for name, embedding in zip(batch_names, embeddings):
                concepts = name_concept_map[name]
                
                point = PointStruct(
                    id=str(uuid.uuid4()),
                    vector=embedding,
                    payload={
                        "concept_name": name,  # Original case from first concept
                        "concept_name_lower": name.lower(),
                        "concepts": [self.concept_to_payload(c) for c in concepts]
                    }
                )
                all_points.append(point)
        
        # Upload to Qdrant in batches
        print(f"Uploading {len(all_points)} points to Qdrant...")
        for i in tqdm(range(0, len(all_points), self.QDRANT_BATCH_SIZE), desc="Uploading"):
            batch = all_points[i:i + self.QDRANT_BATCH_SIZE]
            self.qdrant_client.upsert(
                collection_name=collection_name,
                points=batch
            )
        
        print(f"Successfully uploaded {len(all_points)} points to '{collection_name}'")
    
    def build_meddra_collection(self):
        """Build the main 'meddra' collection from concept names."""
        print("\n" + "="*60)
        print("Building 'meddra' collection")
        print("="*60 + "\n")
        
        # Fetch concepts
        concepts = self.fetch_concepts()
        
        # Group by concept name
        name_concept_map = self.group_concepts_by_name(
            [(c.concept_name, c) for c in concepts]
        )
        print(f"Grouped into {len(name_concept_map)} unique concept names")
        
        # Create collection and upload
        self.create_collection("meddra")
        self.upload_points("meddra", name_concept_map)
    
    def build_synonyms_collection(self):
        """Build the 'synonyms' collection from concept synonyms."""
        print("\n" + "="*60)
        print("Building 'synonyms' collection")
        print("="*60 + "\n")
        
        # Fetch synonyms
        synonyms = self.fetch_synonyms()
        
        # Group by synonym name
        name_concept_map = self.group_concepts_by_name(synonyms)
        print(f"Grouped into {len(name_concept_map)} unique synonym names")
        
        # Create collection and upload
        self.create_collection("synonyms")
        self.upload_points("synonyms", name_concept_map)
    
    def build_all(self):
        """Build both collections."""
        self.build_meddra_collection()
        self.build_synonyms_collection()
        print("\n" + "="*60)
        print("✓ All collections built successfully!")
        print("="*60)


def main():
    """Main entry point."""
    # Load configuration from environment
    config = {
    }
    
    if not config["pg_password"]:
        print("Warning: PG_PASSWORD is empty", file=sys.stderr)
    
    print("Configuration:")
    print(f"  PostgreSQL: {config['pg_user']}@{config['pg_host']}:{config['pg_port']}/{config['pg_dbname']}")
    print(f"  Vocab Schema: {config['vocab_schema']}")
    print(f"  Qdrant URL: {config['qdrant_url']}")
    print(f"  Embedding Model: BAAI/bge-large-en-v1.5 (Local)
        print("Warning: PG_PASSWORD is empty", file=sys.stderr)
    
    print("Configuration:")
    print(f"  PostgreSQL: {config['pg_user']}@{config['pg_host']}:{config['pg_port']}/{config['pg_dbname']}")
    print(f"  Vocab Schema: {config['vocab_schema']}")
    print(f"  Qdrant URL: {config['qdrant_url']}")
    print(f"  OpenAI API Key: {'*' * 20}{config['openai_api_key'][-4:]}")
    print()
    
    # Build collections
    builder = CollectionBuilder(**config)
    builder.build_all()


if __name__ == "__main__":
    main()
