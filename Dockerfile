# Stage 1: Builder
FROM rust:1-slim as builder

# Install build dependencies if any
# RUN apt-get update && apt-get install -y some-build-tool

WORKDIR /usr/src/hecate

# Copy the API source code
COPY api/ ./api/

# Build the release binary and the write_pairs binary
WORKDIR /usr/src/hecate/api
RUN cargo build --release
RUN cargo build --bin write_pairs

# The write_pairs binary will be run at runtime in the final image,
# but we build it here.

# Stage 2: Runtime
FROM debian:stable-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binaries from the builder stage
COPY --from=builder /usr/src/hecate/api/target/release/hecate-api /usr/local/bin/hecate-api
COPY --from=builder /usr/src/hecate/api/target/release/write_pairs /usr/local/bin/write_pairs

WORKDIR /app

# Create an empty ConceptRecordCounts.json file.
# The user should mount a real one if needed.
RUN echo '{}' > ConceptRecordCounts.json

# Set environment variables with placeholder values.
# The user MUST override these at runtime.
ENV SERVER_ADDR="0.0.0.0:8080"
ENV QDRANT_URI="http://qdrant:6334"
ENV VECTORDB_DATA_PATH="/app/all_pairs.txt"
ENV CORS_ORIGINS="http://localhost:5173"
ENV PG__HOST="db"
ENV PG__PORT="5432"
ENV PG__USER="postgres"
ENV PG__PASSWORD="password"
ENV PG__DBNAME="hecate"
ENV VOCAB_SCHEMA="vocab_27_feb_2026"
ENV OPENAI_API_KEY="your-openai-api-key"
ENV UMLS_API_KEY="your-umls-api-key"
ENV RUST_LOG="info"

# Expose the API port
EXPOSE 8080

# The entrypoint will be a script to first run write_pairs and then the api
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
ENTRYPOINT ["docker-entrypoint.sh"]

CMD ["hecate-api"]
