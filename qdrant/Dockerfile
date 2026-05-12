FROM qdrant/qdrant:latest

# Copy collection snapshots if they exist
# These will be placed in /qdrant/storage/ and loaded on startup
COPY collections/ /qdrant/storage/

# Copy custom Qdrant configuration (optional)
# COPY config/production.yaml /qdrant/config/production.yaml

# Expose Qdrant ports
# 6333 - HTTP API
# 6334 - gRPC API (used by Hecate API)
EXPOSE 6333 6334

# Set storage path
ENV QDRANT__STORAGE__STORAGE_PATH=/qdrant/storage

# The qdrant/qdrant image already has an entrypoint
# Collections in /qdrant/storage/ are automatically loaded on startup
