# Hecate UI

A semantic search engine for the OHDSI vocabulary.
Hecate provides an intuitive interface for searching and exploring medical concepts using LLM embeddings and cosine
similarity to deliver relevant search results.

## Features

- **Semantic Search** - Intelligent concept search with autocomplete
- **Advanced Filtering** - Multi-dimensional filtering by domain, vocabulary, concept class, validity, and standard
  concept status
- **Concept Exploration** - Detailed concept views with comprehensive metadata
- **Hierarchical Navigation** - Browse concept relationships and hierarchies
- **Related Concepts** - Discover related medical concepts and PHOEBE relationships

## Technology Stack

- **Frontend**: React 19, TypeScript, Vite
- **UI Components**: Ant Design, Material-UI Icons
- **API**: Axios for HTTP requests
- **Styling**: Emotion, CSS-in-JS
- **Development**: ESLint, Prettier

## Getting Started

### Prerequisites

- Node.js (v18 or higher)
- npm or yarn

### Installation

```bash
# Install dependencies
npm install
```

### Development

```bash
# Start development server
npm run dev
```

The application will be available at `http://localhost:5173`

### Build

```bash
# Build for production
npm run build

# Preview production build
npm run preview
```

### Code Quality

```bash
# Run linting
npm run lint

# Format code
npx prettier --write .
```

## Configuration

### API Endpoints

The application connects to:

- **Primary API**: `https://hecate.pantheon-hds.com/api`
- **Secondary API**: `https://hecate.pantheon-hds.com/v2`

To use local development endpoints, update the configuration in `src/config/api.ts`:

```typescript
export const API_BASE_URL = "http://localhost:8080/api";
export const API_V2_BASE_URL = "http://localhost:8081/v2";
```

## Project Structure

```
src/
├── @types/          # TypeScript type definitions
├── components/      # React components
├── config/          # API configuration
├── service/         # API service layer
└── assets/          # Static assets
```

## Key Components

- **ConceptTable** - Main search results with filtering and expandable rows
- **ConceptView** - Detailed concept display with tabbed interface
- **ConceptHierarchyTable** - Hierarchical concept browsing
- **RelatedConceptsView** - Related concepts and relationships

## Data Models

The application works with several core data types:

- **Concept** - Core medical concept with metadata
- **ConceptRow** - Aggregated concept data for table display
- **RelatedConcept** - Concept relationships and connections
- **SearchResponse** - Search results with scoring
