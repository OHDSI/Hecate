export type Concept = {
  concept_id: number;
  concept_name: string;
  domain_id: string;
  vocabulary_id: string;
  concept_class_id: string;
  concept_code: string;
  standard_concept?: string;
  invalid_reason?: string;
  valid_start_date: string;
  valid_end_date: string;
};

export type RelatedConcept = {
  relationship_id: string;
  concept_id: number;
  concept_name: string;
  vocabulary_id: string;
};

export type SearchResponse = {
  concept_name: string;
  concept_name_lower: string;
  score: number;
  concepts: Concept[];
};

export type ConceptRow = {
  concept_id?: number;
  concept_name: string;
  domain_id: string[];
  vocabulary_id: string[];
  concept_class_id: string[];
  concept_code?: string;
  standard_concept: string[];
  invalid_reason: string[];
  score: number;
  children?: ConceptRow[];
};

export type ConceptExpandRow = {
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
};
