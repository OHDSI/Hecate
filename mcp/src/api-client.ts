import axios, { AxiosInstance, AxiosResponse } from "axios";
import {
  Concept,
  RelatedConcept,
  SearchResponse,
  ConceptExpandRow,
  HecateApiConfig,
  SearchOptions,
} from "./types.js";

export class HecateApiClient {
  private client: AxiosInstance;
  private v2Client: AxiosInstance;
  private config: HecateApiConfig;

  constructor(config: HecateApiConfig) {
    this.config = config;

    this.client = axios.create({
      baseURL: config.baseUrl,
      timeout: config.timeout || 10000,
      headers: {
        "Content-Type": "application/json",
      },
    });

    this.v2Client = axios.create({
      baseURL: config.v2BaseUrl,
      timeout: config.timeout || 10000,
      headers: {
        "Content-Type": "application/json",
      },
    });
  }

  async search(query: string, options?: SearchOptions): Promise<SearchResponse[]> {
    const params = new URLSearchParams({ q: query });
    
    if (options?.vocabulary_id) {
      params.append('vocabulary_id', options.vocabulary_id);
    }
    if (options?.standard_concept !== undefined) {
      params.append('standard_concept', options.standard_concept);
    }
    if (options?.domain_id) {
      params.append('domain_id', options.domain_id);
    }
    if (options?.concept_class_id) {
      params.append('concept_class_id', options.concept_class_id);
    }
    if (options?.limit) {
      params.append('limit', options.limit.toString());
    }
    
    const response: AxiosResponse<SearchResponse[]> = await this.client.get(
      `/search?${params.toString()}`,
    );
    return response.data;
  }

  async getConceptById(id: number): Promise<Concept> {
    const response: AxiosResponse<Concept[]> = await this.client.get(
      `/concepts/${id}`,
    );
    return response.data[0];
  }

  async getConceptRelationships(id: number): Promise<RelatedConcept[]> {
    const response: AxiosResponse<RelatedConcept[]> = await this.client.get(
      `/concepts/${id}/relationships`,
    );
    return response.data;
  }

  async getConceptPhoebe(id: number): Promise<RelatedConcept[]> {
    const response: AxiosResponse<RelatedConcept[]> = await this.client.get(
      `/concepts/${id}/phoebe`,
    );
    return response.data;
  }

  async getConceptDefinition(id: number): Promise<string> {
    const response: AxiosResponse<string> = await this.client.get(
      `/concepts/${id}/definition`,
    );
    return response.data;
  }

  async expandConcept(
    id: number,
    childLevels: number = 5,
    parentLevels: number = 0,
  ): Promise<ConceptExpandRow[]> {
    const response: AxiosResponse<{ concepts: ConceptExpandRow[] }> =
      await this.v2Client.get(
        `/concepts/${id}/expand?childlevels=${childLevels}&parentlevels=${parentLevels}`,
      );

    if (response.data.concepts[0]?.children) {
      return response.data.concepts[0].children;
    } else {
      return [];
    }
  }
}
