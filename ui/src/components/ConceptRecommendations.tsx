import { useMemo } from "react";
import { Typography } from "antd";
import {
  ConceptRecommendations,
  RecommendedConcept,
} from "../service/conceptSets";
import { ConceptRow } from "../@types/data-source";
import ConceptTableCore from "./ConceptTableCore";
import {
  expandInvalidReasonAbbreviation,
  expandStandardConceptAbbreviation,
} from "../service/search";

const { Title, Text } = Typography;

interface ConceptRecommendationsProps {
  recommendations: ConceptRecommendations;
}

const ConceptRecommendationsComponent: React.FC<
  ConceptRecommendationsProps
> = ({ recommendations }) => {
  // Create initial filters for pre-selection
  const initialFilters = useMemo(() => {
    const filters: any = {};
    
    // Pre-select all returned vocabularies
    if (recommendations.used_vocabularies && recommendations.used_vocabularies.length > 0) {
      filters.vocabulary_id = recommendations.used_vocabularies;
    }
    
    // Pre-select only standard concepts
    filters.standard_concept = ["Standard"];
    
    return filters;
  }, [recommendations.used_vocabularies]);

  // Transform recommendations data to match ConceptRow format used in main search table
  const transformedData: ConceptRow[] = useMemo(() => {
    return recommendations.recommendations.map((rec: RecommendedConcept) => ({
      concept_id: rec.concept_id,
      concept_name: rec.concept_name,
      domain_id: [rec.domain_id],
      vocabulary_id: [rec.vocabulary_id],
      concept_class_id: [rec.concept_class_id],
      concept_code: rec.concept_code,
      standard_concept: [
        expandStandardConceptAbbreviation(rec.standard_concept),
      ],
      invalid_reason: [
        expandInvalidReasonAbbreviation(rec.invalid_reason || undefined),
      ],
      score: rec.similarity_score,
    }));
  }, [recommendations.recommendations]);

  if (!recommendations || !recommendations.recommendations.length) {
    return (
      <div
        style={{
          textAlign: "center",
          paddingTop: "1em",
          paddingLeft: "2em",
          paddingRight: "2em",
          color: "#999",
        }}
      >
        <Text type="secondary">No recommendations available</Text>
      </div>
    );
  }

  return (
    <div>
      <div
        style={{
          marginBottom: "1em",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <Title level={5} style={{ margin: 0 }}>
          Recommendations{" "}
        </Title>
      </div>

      <ConceptTableCore
        data={transformedData}
        full={true}
        showFilters={true}
        loading={false}
        hiddenColumns={["concept_code", "invalid_reason"]}
        initialFilters={initialFilters}
      />

      <div style={{ marginTop: "0.5em", fontSize: "11px", color: "#666" }}>
        <Text type="secondary">
          These are standard concepts similar to your included concepts.
        </Text>
      </div>
    </div>
  );
};

export default ConceptRecommendationsComponent;
