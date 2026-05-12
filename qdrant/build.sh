#!/bin/bash
#
# Build Qdrant collections for Hecate
# This script sets up a Python virtual environment and runs the collection builder
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "Hecate Qdrant Collection Builder"
echo "=========================================="
echo

# Check if .env exists
if [ ! -f .env ]; then
    echo "❌ Error: .env file not found"
    echo
    echo "Please create a .env file with your configuration:"
    echo "  cp config.template .env"
    echo
    echo "Then edit .env with your actual PostgreSQL, Qdrant, and OpenAI credentials."
    exit 1
fi

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "📦 Creating Python virtual environment..."
    python3 -m venv venv
    echo "✓ Virtual environment created"
    echo
fi

# Activate virtual environment
echo "🔧 Activating virtual environment..."
source venv/bin/activate

# Install/upgrade dependencies
echo "📥 Installing dependencies..."
pip install -q --upgrade pip
pip install -q -r requirements.txt
echo "✓ Dependencies installed"
echo

# Run the builder
echo "🚀 Starting collection builder..."
echo
python build_collections.py

echo
echo "=========================================="
echo "✓ Build complete!"
echo "=========================================="
echo
echo "Next steps:"
echo "  1. Copy the Qdrant storage to ./collections/:"
echo "     cp -r /path/to/qdrant/storage/* ./collections/"
echo
echo "  2. Return to project root and build Docker images:"
echo "     cd .."
echo "     ./build.sh"
