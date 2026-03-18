# Hecate MCP Server

A Model Context Protocol (MCP) server for the [Hecate API](https://hecate.pantheon-hds.com), providing access to OMOP vocabulary concepts.

## Features

- **Search concepts** — semantic search across OMOP vocabularies (SNOMED, ICD-10, RxNorm, LOINC, and more)
- **Lookup by ID** — retrieve full concept metadata by OMOP concept ID
- **Relationships** — explore cross-vocabulary mappings and standard concept relationships
- **PHOEBE relationships** — curated clinical relationships optimised for phenotype and cohort definitions
- **Hierarchy expansion** — navigate concept hierarchies up and down

## Setup

### Hosted

Connect directly using the hosted server at `https://hecate.pantheon-hds.com/mcp/sse` (Streamable HTTP — the `/sse` path is retained for backwards compatibility) no install of the MCP server required. Refer to your client's MCP documentation for how to add a remote server URL:

- [Claude Desktop](https://modelcontextprotocol.io/quickstart/user)
- [Cursor](https://docs.cursor.com/context/model-context-protocol)
- [Windsurf](https://docs.windsurf.com/windsurf/cascade/mcp)
- [opencode](https://opencode.ai/docs/mcp-servers/)
- [ChatGPT](https://help.openai.com/en/articles/11487775-apps-in-chatgpt) (Plus, Team, or Pro)

### Claude Desktop (drag and drop)

Download [`hecate-mcp-server.mcpb`](./hecate-mcp-server.mcpb) and drag it into the Claude Desktop settings to install instantly.

### Local

1. Clone the repository and install dependencies:
   ```bash
   npm install && npm run build
   ```
2. Register `dist/index.js` as a local stdio MCP server in your client using the same docs above.

## Development

Built with TypeScript, MCP SDK, Axios, and Zod.

```
src/
├── index.ts       # MCP server and tool definitions
├── api-client.ts  # Hecate API client
└── types.ts       # TypeScript types
```

```bash
npm run dev    # run with tsx (no build needed)
npm run build  # compile to dist/
npm start      # run compiled output
```

By default connects to `https://hecate.pantheon-hds.com/api`. To use a different endpoint, update `DEFAULT_CONFIG` in `src/index.ts`.
