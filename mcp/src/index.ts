#!/usr/bin/env node

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ErrorCode,
  ListToolsRequestSchema,
  McpError,
} from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { HecateApiClient } from "./api-client.js";
import { HecateApiConfig } from "./types.js";

// Default configuration
const DEFAULT_CONFIG: HecateApiConfig = {
  baseUrl: "https://hecate.pantheon-hds.com/api",
  v2BaseUrl: "https://hecate.pantheon-hds.com/v2",
  timeout: 10000,
};

// Create API client
const apiClient = new HecateApiClient(DEFAULT_CONFIG);

// Create MCP server
const server = new Server(
  {
    name: "hecate-mcp-server",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
    },
  },
);

// Input validation schemas
const SearchInputSchema = z.object({
  query: z.string().min(1).max(500),
  vocabulary_id: z.string().optional(),
  standard_concept: z.string().optional(),
  domain_id: z.string().optional(),
  concept_class_id: z.string().optional(),
  limit: z.number().int().min(1).max(150).optional(),
});

const ConceptIdInputSchema = z.object({
  id: z.number().int().positive(),
});

const AutocompleteInputSchema = z.object({
  query: z.string().min(1).max(100),
});

const ExpandConceptInputSchema = z.object({
  id: z.number().int().positive(),
  childLevels: z.number().int().min(0).max(10).optional(),
  parentLevels: z.number().int().min(0).max(10).optional(),
});

// Tool definitions
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "search_concepts",
        description:
          "Search for OMOP Standardized Vocabulary concepts using text query. Returns medical concepts with similarity scores from over 100 vocabularies including SNOMED CT, ICD-10-CM, RxNorm, LOINC, and more. Concepts are standardized for observational research and cover conditions, drugs, procedures, measurements, and other clinical entities.",
        inputSchema: {
          type: "object",
          properties: {
            query: {
              type: "string",
              description:
                "Search query for medical concepts. Can search by concept name, ID, or code. Examples: 'diabetes', 'hypertension', '4321435', 'E11.9'.",
              minLength: 1,
              maxLength: 500,
            },
            vocabulary_id: {
              type: "string",
              description: "Filter by vocabulary source. Common vocabularies: 'SNOMED', 'ICD10CM', 'RxNorm', 'LOINC', 'CPT4', 'HCPCS', 'MedDRA'. Supports multiple values as comma-separated string.",
            },
            standard_concept: {
              type: "string",
              description: "Filter by standardization status: 'S' = Standard concepts, 'C' = Classification concepts, empty/null = Non-standard source concepts.",
            },
            domain_id: {
              type: "string",
              description: "Filter by clinical domain: 'Condition', 'Drug', 'Procedure', 'Measurement', 'Observation', 'Device', 'Visit'. Supports multiple values as comma-separated string.",
            },
            concept_class_id: {
              type: "string",
              description: "Filter by concept classification within vocabulary. Examples: 'Clinical Finding', 'Ingredient', 'Procedure', 'Lab Test'. Supports multiple values as comma-separated string.",
            },
            limit: {
              type: "number",
              description: "Maximum number of results to return (default: 25, max: 150)",
              minimum: 1,
              maximum: 150,
            },
          },
          required: ["query"],
        },
      },
      {
        name: "get_concept_by_id",
        description:
          "Get detailed information about a specific OMOP concept by its unique concept ID. Returns concept metadata including name, domain, vocabulary, concept class, and validity dates.",
        inputSchema: {
          type: "object",
          properties: {
            id: {
              type: "number",
              description: "OMOP concept ID - unique identifier for the concept",
              minimum: 1,
            },
          },
          required: ["id"],
        },
      },
      {
        name: "get_concept_relationships",
        description: "Get related concepts for a specific OMOP concept. Returns relationships like 'Maps to', 'Subsumes', 'Is a', enabling navigation through concept hierarchies and cross-vocabulary mappings.",
        inputSchema: {
          type: "object",
          properties: {
            id: {
              type: "number",
              description: "OMOP concept ID to find relationships for",
              minimum: 1,
            },
          },
          required: ["id"],
        },
      },
      {
        name: "get_concept_phoebe",
        description:
          "Get PHOEBE-defined relationships for a specific OMOP concept. PHOEBE provides curated clinical relationships optimized for phenotype definitions and cohort building.",
        inputSchema: {
          type: "object",
          properties: {
            id: {
              type: "number",
              description: "OMOP concept ID to get PHOEBE relationships for",
              minimum: 1,
            },
          },
          required: ["id"],
        },
      },
      {
        name: "get_concept_definition",
        description:
          "Get the clinical definition from UMLS for a specific OMOP concept if available. Returns detailed descriptions to help understand the medical meaning and clinical context.",
        inputSchema: {
          type: "object",
          properties: {
            id: {
              type: "number",
              description: "OMOP concept ID to get definition for",
              minimum: 1,
            },
          },
          required: ["id"],
        },
      },
      {
        name: "expand_concept_hierarchy",
        description:
          "Get the hierarchical structure of an OMOP concept including children and parents. Useful for exploring clinical taxonomies and building comprehensive phenotype definitions.",
        inputSchema: {
          type: "object",
          properties: {
            id: {
              type: "number",
              description: "OMOP concept ID to expand hierarchy for",
              minimum: 1,
            },
            childLevels: {
              type: "number",
              description: "Number of child levels to expand (default: 5)",
              minimum: 0,
              maximum: 10,
            },
            parentLevels: {
              type: "number",
              description: "Number of parent levels to expand (default: 0)",
              minimum: 0,
              maximum: 10,
            },
          },
          required: ["id"],
        },
      },
    ],
  };
});

// Tool execution handler
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "search_concepts": {
        const { query, vocabulary_id, standard_concept, domain_id, concept_class_id, limit = 25 } = SearchInputSchema.parse(args);
        const results = await apiClient.search(query, {
          vocabulary_id,
          standard_concept,
          domain_id,
          concept_class_id,
          limit,
        });
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(results, null, 2),
            },
          ],
        };
      }

      case "get_concept_by_id": {
        const { id } = ConceptIdInputSchema.parse(args);
        const concept = await apiClient.getConceptById(id);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(concept, null, 2),
            },
          ],
        };
      }

      case "get_concept_relationships": {
        const { id } = ConceptIdInputSchema.parse(args);
        const relationships = await apiClient.getConceptRelationships(id);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(relationships, null, 2),
            },
          ],
        };
      }

      case "get_concept_phoebe": {
        const { id } = ConceptIdInputSchema.parse(args);
        const phoebe = await apiClient.getConceptPhoebe(id);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(phoebe, null, 2),
            },
          ],
        };
      }

      case "get_concept_definition": {
        const { id } = ConceptIdInputSchema.parse(args);
        const definition = await apiClient.getConceptDefinition(id);
        return {
          content: [
            {
              type: "text",
              text: definition,
            },
          ],
        };
      }

      case "expand_concept_hierarchy": {
        const {
          id,
          childLevels = 5,
          parentLevels = 0,
        } = ExpandConceptInputSchema.parse(args);
        const hierarchy = await apiClient.expandConcept(
          id,
          childLevels,
          parentLevels,
        );
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(hierarchy, null, 2),
            },
          ],
        };
      }

      default:
        throw new McpError(ErrorCode.MethodNotFound, `Unknown tool: ${name}`);
    }
  } catch (error) {
    if (error instanceof z.ZodError) {
      throw new McpError(
        ErrorCode.InvalidParams,
        `Invalid parameters: ${error.message}`,
      );
    }

    if (error instanceof McpError) {
      throw error;
    }

    // Handle API errors
    if (error && typeof error === "object" && "response" in error) {
      const axiosError = error as any;
      throw new McpError(
        ErrorCode.InternalError,
        `API Error: ${axiosError.response?.status} - ${axiosError.response?.statusText || axiosError.message}`,
      );
    }

    throw new McpError(
      ErrorCode.InternalError,
      `Unexpected error: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
});

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
