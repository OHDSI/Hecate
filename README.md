# Hecate

![./ui/public/screenshot.png](./ui/public/screenshot.png)

**Hecate is a semantic search engine for the OHDSI vocabulary**

Try it out online at [https://hecate.pantheon-hds.com](https://hecate.pantheon-hds.com)

or use the API [https://hecate.pantheon-hds.com/openapi/#/](https://hecate.pantheon-hds.com/openapi/#/)

## Overview

Hecate consists of three main components:

- **hecate-ui** - React-based frontend interface for semantic search
- **hecate-api** - Rust API backend for concept data and search
- **autocomplete** - Rust autocomplete service for search suggestions

## Used by

- [Ariadne](https://ohdsi.github.io/Ariadne/) — uses the Hecate API for semantic search

## Running locally

See the [API local installation guide](./api/LOCAL_INSTALL.md) for setting up PostgreSQL, Qdrant, and the required data files. Once that's done:

```bash
# API
cd api
cargo run

# Frontend
cd ui
npm install
npm run dev

# Autocomplete
cd autocomplete
cargo run
```

## MCP Server

An MCP (Model Context Protocol) server is available for integration with MCP-compatible tools.
You can connect your LLM with https://hecate.pantheon-hds.com/mcp/sse to try it out.
To build and run locally see the [mcp/README.md](./mcp/README.md) for more details.
