#!/bin/sh
set -e

# Run write_pairs to generate the concept index.
# This requires Qdrant to be running and accessible.
# In a real setup, you might want to add a wait-for-it script here.
echo "Generating concept index..."
/usr/local/bin/write_pairs

echo "Starting Hecate API..."

# Now, execute the main command (passed from CMD)
exec "$@"
