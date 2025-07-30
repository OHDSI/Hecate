import { createApiClient } from "../config/api";

export interface RecommendedConcept {
  concept_id: number;
  concept_name: string;
  vocabulary_id: string;
  domain_id: string;
  concept_class_id: string;
  concept_code: string;
  standard_concept: string;
  invalid_reason: string | null;
  similarity_score: number;
  source_concept_id: number;
}

export interface ConceptRecommendations {
  recommendations: RecommendedConcept[];
  total_count: number;
  used_vocabularies: string[];
}

export interface AnalysisResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
  concept_summary?: {
    included_concepts_count: number;
    included_descendants_count: number;
    included_mapped_count: number;
    excluded_concepts_count: number;
    excluded_descendants_count: number;
    excluded_mapped_count: number;
    total_included: number;
    total_excluded: number;
  };
  recommendations?: ConceptRecommendations;
}

export const analyzeConceptSet = async (
  conceptSet: string,
): Promise<AnalysisResult> => {
  const client = createApiClient();

  return client
    .post<AnalysisResult>("/conceptsets/analyze", {
      concept_set: conceptSet,
    })
    .then((resp) => {
      return resp.data;
    })
    .catch((err) => {
      throw err;
    });
};
