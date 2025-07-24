export interface Concept {
  concept_id: number;
  concept_name: string;
  domain_id: string;
  vocabulary_id: string;
  concept_class_id: string;
  concept_code: string;
  standard_concept?: string;
  invalid_reason?: string;
  valid_start_date?: string;
  valid_end_date?: string;
}

export interface RelatedConcept {
  relationship_id: string;
  concept_id: number;
  concept_name: string;
  vocabulary_id: string;
}

export interface SearchResponse {
  concept_name: string;
  concept_name_lower: string;
  score?: number;
  concepts: Concept[];
}

export interface ConceptExpandRow {
  concept_id: number;
  concept_name: string;
  domain_id: string;
  vocabulary_id: string;
  concept_class_id: string;
  concept_code: string;
  standard_concept: string;
  invalid_reason: string;
  level: number;
  children?: ConceptExpandRow[];
}

export interface SearchOptions {
  vocabulary_id?: string;
  standard_concept?: string;
  domain_id?: string;
  concept_class_id?: string;
  limit?: number;
}

export interface HecateApiConfig {
  baseUrl: string;
  v2BaseUrl: string;
  timeout?: number;
}
