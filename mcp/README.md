# Hecate MCP Server

A Model Context Protocol (MCP) server for the Hecate API, providing access to medical concept search and retrieval functionality.

## Features

This MCP server provides the following tools:

### `hecate_search_concepts`
Search for medical concepts using text queries with similarity scoring.

**Parameters:**
- `query` (string): Search query for medical concepts (1-500 characters)
- `vocabulary_id` (string, optional): Filter by vocabulary source (e.g., 'SNOMED', 'ICD10CM', 'RxNorm')
- `standard_concept` (string, optional): Filter by standardization status ('S', 'C', or empty)
- `domain_id` (string, optional): Filter by clinical domain (e.g., 'Condition', 'Drug')
- `concept_class_id` (string, optional): Filter by concept class
- `limit` (number, optional): Maximum results to return (1-50, default: 20)

**Returns:** Array of search results with concept details and similarity scores

### `hecate_get_concept_by_id`
Get detailed information about a specific medical concept by its ID.

**Parameters:**
- `id` (number): Concept ID (positive integer)

**Returns:** Detailed concept information including all metadata

### `hecate_get_concept_relationships`
Get related concepts for a specific concept ID.

**Parameters:**
- `id` (number): Concept ID (positive integer)

**Returns:** Array of related concepts with relationship types

### `hecate_get_concept_phoebe`
Get PHOEBE-defined relationships for a specific concept ID.

**Parameters:**
- `id` (number): Concept ID (positive integer)

**Returns:** Array of PHOEBE-related concepts

### `hecate_expand_concept_hierarchy`
Get the hierarchical structure of a concept including children and parents.

**Parameters:**
- `id` (number): Concept ID (positive integer)
- `childLevels` (number, optional): Number of child levels to expand (0-10, default: 5)
- `parentLevels` (number, optional): Number of parent levels to expand (0-10, default: 0)

**Returns:** Hierarchical structure with nested concepts

## Installation

1. Clone the repository
2. Install dependencies:
   ```bash
   npm install
   ```

3. Build the server:
   ```bash
   npm run build
   ```

## Usage

### Development
```bash
npm run dev
```

### Production
```bash
npm start
```

### With Claude Desktop

Add the server to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "hecate": {
      "command": "node",
      "args": ["/path/to/hecate-mcp-server/dist/index.js"],
      "env": {}
    }
  }
}
```

## API Configuration

By default, the server connects to:
`https://hecate.pantheon-hds.com/api`

To use a different API endpoint, modify the `DEFAULT_CONFIG` in `src/index.ts`.

## Development

The server is built with:
- TypeScript
- Model Context Protocol SDK
- Axios for HTTP requests
- Zod for input validation

### Project Structure

```
src/
├── index.ts          # Main MCP server implementation
├── api-client.ts     # Hecate API client
└── types.ts          # TypeScript type definitions
```

### Building

```bash
npm run build
```

### Testing

```bash
npm test
```
