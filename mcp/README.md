# Hecate MCP Server

A Model Context Protocol (MCP) server for the Hecate API, providing access to medical concept search and retrieval functionality.

## Features

This MCP server provides the following tools:

### `search_concepts`
Search for medical concepts using text queries with similarity scoring.

**Parameters:**
- `query` (string): Search query for medical concepts (1-500 characters)

**Returns:** Array of search results with concept details and similarity scores

### `autocomplete_concepts`
Get autocomplete suggestions for medical concept names.

**Parameters:**
- `query` (string): Partial concept name for autocomplete (1-100 characters)

**Returns:** Array of suggested concept names

### `get_concept_by_id`
Get detailed information about a specific medical concept by its ID.

**Parameters:**
- `id` (number): Concept ID (positive integer)

**Returns:** Detailed concept information including all metadata

### `get_concept_relationships`
Get related concepts for a specific concept ID.

**Parameters:**
- `id` (number): Concept ID (positive integer)

**Returns:** Array of related concepts with relationship types

### `get_concept_phoebe`
Get phoebe-related concepts for a specific concept ID.

**Parameters:**
- `id` (number): Concept ID (positive integer)

**Returns:** Array of phoebe-related concepts

### `get_concept_definition`
Get the definition of a specific medical concept.

**Parameters:**
- `id` (number): Concept ID (positive integer)

**Returns:** Text definition of the concept

### `expand_concept_hierarchy`
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
- Main API: `https://hecate.pantheon-hds.com/api`
- V2 API: `https://hecate.pantheon-hds.com/v2`

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

## Error Handling

The server includes comprehensive error handling for:
- Invalid input parameters (validated with Zod)
- API connection errors
- HTTP response errors
- Unexpected server errors

All errors are properly formatted as MCP errors with appropriate error codes.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License