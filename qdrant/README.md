# Qdrant Collections Builder

This directory contains scripts to build the Qdrant vector collections for Hecate.

## Collections

Two collections are created:

1. **`meddra`** - Main concept collection indexed by concept names
2. **`synonyms`** - Synonym collection indexed by concept synonym names

Both collections use the **BAAI/bge-large-en-v1.5** model with 1024 dimensions, running locally.

## Prerequisites

- PostgreSQL database with OHDSI vocabulary loaded
- Qdrant instance running (local or remote)
- Python 3.8+
- **No API key required** - embeddings run locally on your machine

## Installation

```bash
cd qdrant
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
```

**Note:** On first run, the BAAI embedding model (~1.3GB) will be automatically downloaded to `~/.cache/huggingface/`. This is a one-time download.

## Configuration

### Option 1: Using .env file (Recommended)

Copy the configuration template and edit it:

```bash
cp config.template .env
# Then edit .env with your actual values
```

The script will automatically load the `.env` file if `python-dotenv` is installed.

### Option 2: Environment Variables

Alternatively, set environment variables directly:

```bash
# PostgreSQL connection
export PG_HOST=localhost
export PG_PORT=5432
export PG_USER=postgres
export PG_PASSWORD=your_password
export PG_DBNAME=hecate
export VOCAB_SCHEMA=vocab_27_feb_2026

# OpenAI API key (required)
export OPENAI_API_KEY=sk-your-key-here
```

## Usage

### Quick Start with build.sh (Recommended)

```bash
cd qdrant

# Create and configure .env file
cp config.template .env
# Edit .env with your actual credentials

# Run the build script (handles venv setup and dependencies)
./build.sh
```

### Manual Build

Build all collections manually:

```bash
cd qdrant

# Setup
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Configure
cp config.template .env
# Edit .env with your credentials

# Build
python build_collections.py
```

This will:
1. Connect to PostgreSQL and fetch all concepts and synonyms
2. Generate embeddings using OpenAI (this will incur API costs ~$2-3)
3. Load the BAAI/bge-large-en-v1.5 model (downloads on first run)
3. Generate embeddings locally using your CPU/GPU
4. Create the `meddra` and `synonyms` collections in Qdrant
5
### Build Individual Collections

You can modify the script to build only one collection by commenting out the other in the `build_all()` method.

## Cost Estimation

ThePerformance

### M1/M2/M3 Mac (Apple Silicon)

- **Speed**: ~500-1000 embeddings/second with MPS acceleration
- **Time**: ~20-40 minutes for full OHDSI vocabulary (~1M concepts)
- **Memory**: ~3-4GB RAM during processing
- **Cost**: **FREE** - completely local, no API costs

### Intel Mac / Linux

- **Speed**: ~200-500 embeddings/second (CPU only)
- **Time**: ~40-90 minutes for full vocabulary
- **Memory**: ~2-3GB RAM
- **Cost**: **FREE** - no API costs

The script automatically detects and uses:
- **MPS (Metal)** on Apple Silicon Macs for GPU acceleration
- **CPU** on other systems

## Cost Estimation

**Total cost: $0.00** - The BAAI model runs entirely on your local machine with no API calls.

The only "cost" is the one-time download of the model (~1.3GB) and compute time

After running the script, the collections will be stored in Qdrant. To copy them for Docker deployment:

```bash
# Find your Qdrant storage directory
# Default locations:
# - Docker: /qdrant/storage
# - Local: ./storage (relative to where Qdrant was started)

# Copy collections to docker build directory
cp -r /path/to/qdrant/storage/* ./collections/
```

The collections will then be included in the `hecate-qdrant` Docker image when you run `./build.sh`.

## Data Structure

Each point in both collections has:

- **id**: UUID (v4)
- **vector**: 1024-dimensional embedding
- **payload**:
  - `concept_name`: Original concept/synonym name
  - `concept_name_lower`: Lowercased for exact matching
  - `concepts`: Array of concept objects, each containing: (default: 128).

### Model Download Issues

If the model download fails or is interrupted:
1. Delete the partial download: `rm -rf ~/.cache/huggingface/hub/models--BAAI--bge-large-en-v1.5`
2. Re-run the script to download again

### PostgreSQL Connection Issues

Ensure your PostgreSQL connection details are correct and the vocabulary schema exists with data loaded.

### Qdrant Connection Issues

Ensure Qdrant is running and accessible at the configured URL.

### Slow Performance

- **On Mac**: Ensure you're running on Apple Silicon (M1/M2/M3) for MPS acceleration
- **On Linux**: Consider using a machine with GPU and installing CUDA-enabled PyTorch
- You can reduce `BATCH_SIZE` if memory is constrained, though this will be slightly slower

## Model Information

**BAAI/bge-large-en-v1.5**
- Dimensions: 1024
- License: MIT
- Size: ~1.3GB
- Quality: State-of-the-art for retrieval tasks
- Paper: https://arxiv.org/abs/2309.07597
- Hugging Face: https://huggingface.co/BAAI/bge-large-en-v1.5

This model is specifically designed for semantic search and retrieval tasks, making it ideal for medical concept matching

### Out of Memory

If you run out of memory during embedding generation, reduce the `BATCH_SIZE` constant in the script.

### OpenAI Rate Limits

The script handles batching to avoid overwhelming the OpenAI API, but if you hit rate limits, you may need to add delays between batches.

### PostgreSQL Connection Issues

Ensure your PostgreSQL connection details are correct and the vocabulary schema exists with data loaded.

### Qdrant Connection Issues

Ensure Qdrant is running and accessible at the configured URL.
