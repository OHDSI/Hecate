#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { HecateApiClient } from "./api-client.js";
import { HecateApiConfig } from "./types.js";

// API URLs
// const API_BASE_URL = "http://localhost:8080/api";
const API_BASE_URL = "https://hecate.pantheon-hds.com/api";

// Default configuration
const DEFAULT_CONFIG: HecateApiConfig = {
  baseUrl: API_BASE_URL,
  timeout: 10000,
};

// Create API client
const apiClient = new HecateApiClient(DEFAULT_CONFIG);

// Create MCP server
const server = new McpServer({
  name: "hecate-mcp-server",
  version: "1.0.0",
});

// Shared annotations for all read-only tools
const READ_ONLY_ANNOTATIONS = {
  readOnlyHint: true,
  destructiveHint: false,
  idempotentHint: true,
  openWorldHint: true,
};

// Helper to format API errors as tool results
function apiErrorResult(error: unknown) {
  let message: string;
  if (error && typeof error === "object" && "response" in error) {
    const axiosError = error as any;
    message = `API Error: ${axiosError.response?.status} - ${axiosError.response?.statusText || axiosError.message}`;
  } else {
    message = `Error: ${error instanceof Error ? error.message : String(error)}`;
  }
  return {
    isError: true as const,
    content: [{ type: "text" as const, text: message }],
  };
}

// Tool: hecate_search_concepts
server.registerTool(
  "hecate_search_concepts",
  {
    title: "Search OMOP Concepts",
    description:
      "Search for OMOP Standardized Vocabulary concepts using text query. Returns medical concepts with similarity scores from over 100 vocabularies including SNOMED CT, ICD-10-CM, RxNorm, LOINC, and more. Concepts are standardized for observational research and cover conditions, drugs, procedures, measurements, and other clinical entities.",
    inputSchema: {
      query: z
        .string()
        .min(1)
        .max(500)
        .describe(
          "Search query for medical concepts. Can search by concept name, ID, or code. Examples: 'diabetes', 'hypertension', '4321435', 'E11.9'.",
        ),
      vocabulary_id: z
        .string()
        .optional()
        .describe(
          "Filter by vocabulary source. Common vocabularies: 'SNOMED', 'ICD10CM', 'RxNorm', 'LOINC', 'CPT4', 'HCPCS', 'MedDRA'. Supports multiple values as comma-separated string.",
        ),
      standard_concept: z
        .string()
        .optional()
        .describe(
          "Filter by standardization status: 'S' = Standard concepts, 'C' = Classification concepts, empty/null = Non-standard source concepts.",
        ),
      domain_id: z
        .string()
        .optional()
        .describe(
          "Filter by clinical domain: 'Condition', 'Drug', 'Procedure', 'Measurement', 'Observation', 'Device', 'Visit'. Supports multiple values as comma-separated string.",
        ),
      concept_class_id: z
        .string()
        .optional()
        .describe(
          "Filter by concept classification within vocabulary. Examples: 'Clinical Finding', 'Ingredient', 'Procedure', 'Lab Test'. Supports multiple values as comma-separated string.",
        ),
      limit: z
        .number()
        .int()
        .min(1)
        .max(50)
        .default(20)
        .describe("Maximum number of results to return (default: 20, max: 50)"),
    },
    annotations: READ_ONLY_ANNOTATIONS,
  },
  async ({ query, vocabulary_id, standard_concept, domain_id, concept_class_id, limit }) => {
    try {
      const results = await apiClient.search(query, {
        vocabulary_id,
        standard_concept,
        domain_id,
        concept_class_id,
        limit,
      });
      return { content: [{ type: "text", text: JSON.stringify(results, null, 2) }] };
    } catch (error) {
      return apiErrorResult(error);
    }
  },
);

// Tool: hecate_get_concept_by_id
server.registerTool(
  "hecate_get_concept_by_id",
  {
    title: "Get OMOP Concept by ID",
    description:
      "Get detailed information about a specific OMOP concept by its unique concept ID. Returns concept metadata including name, domain, vocabulary, concept class, and validity dates.",
    inputSchema: {
      id: z
        .number()
        .int()
        .positive()
        .describe("OMOP concept ID - unique identifier for the concept"),
    },
    annotations: READ_ONLY_ANNOTATIONS,
  },
  async ({ id }) => {
    try {
      const concept = await apiClient.getConceptById(id);
      return { content: [{ type: "text", text: JSON.stringify(concept, null, 2) }] };
    } catch (error) {
      return apiErrorResult(error);
    }
  },
);

// Tool: hecate_get_concept_relationships
server.registerTool(
  "hecate_get_concept_relationships",
  {
    title: "Get OMOP Concept Relationships",
    description:
      "Get related concepts for a specific OMOP concept. Returns relationships like 'Maps to', 'Subsumes', 'Is a', enabling navigation through concept hierarchies and cross-vocabulary mappings.",
    inputSchema: {
      id: z
        .number()
        .int()
        .positive()
        .describe("OMOP concept ID to find relationships for"),
    },
    annotations: READ_ONLY_ANNOTATIONS,
  },
  async ({ id }) => {
    try {
      const relationships = await apiClient.getConceptRelationships(id);
      return { content: [{ type: "text", text: JSON.stringify(relationships, null, 2) }] };
    } catch (error) {
      return apiErrorResult(error);
    }
  },
);

// Tool: hecate_get_concept_phoebe
server.registerTool(
  "hecate_get_concept_phoebe",
  {
    title: "Get OMOP Concept PHOEBE Relationships",
    description:
      "Get PHOEBE-defined relationships for a specific OMOP concept. PHOEBE provides curated clinical relationships optimized for phenotype definitions and cohort building.",
    inputSchema: {
      id: z
        .number()
        .int()
        .positive()
        .describe("OMOP concept ID to get PHOEBE relationships for"),
    },
    annotations: READ_ONLY_ANNOTATIONS,
  },
  async ({ id }) => {
    try {
      const phoebe = await apiClient.getConceptPhoebe(id);
      return { content: [{ type: "text", text: JSON.stringify(phoebe, null, 2) }] };
    } catch (error) {
      return apiErrorResult(error);
    }
  },
);

// Tool: hecate_expand_concept_hierarchy
server.registerTool(
  "hecate_expand_concept_hierarchy",
  {
    title: "Expand OMOP Concept Hierarchy",
    description:
      "Get the hierarchical structure of an OMOP concept including children and parents. Useful for exploring clinical taxonomies and building comprehensive phenotype definitions.",
    inputSchema: {
      id: z
        .number()
        .int()
        .positive()
        .describe("OMOP concept ID to expand hierarchy for"),
      childLevels: z
        .number()
        .int()
        .min(0)
        .max(10)
        .default(5)
        .describe("Number of child levels to expand (default: 5)"),
      parentLevels: z
        .number()
        .int()
        .min(0)
        .max(10)
        .default(0)
        .describe("Number of parent levels to expand (default: 0)"),
    },
    annotations: READ_ONLY_ANNOTATIONS,
  },
  async ({ id, childLevels, parentLevels }) => {
    try {
      const hierarchy = await apiClient.expandConcept(id, childLevels, parentLevels);
      return { content: [{ type: "text", text: JSON.stringify(hierarchy, null, 2) }] };
    } catch (error) {
      return apiErrorResult(error);
    }
  },
);

// Start the server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Hecate MCP server running on stdio");
}

main().catch((error) => {
  console.error("Fatal error in main():", error);
  process.exit(1);
});
