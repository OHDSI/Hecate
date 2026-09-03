import { Button, Tag } from "antd";
import type { Key } from "react";

type Filters = Record<string, readonly (Key | boolean)[] | null>;

interface ConceptFilterSummaryProps {
  filters: Filters;
  onClear: () => void;
}

const filterLabels = {
  concept_class_id: "class",
  domain_id: "domain",
  invalid_reason: "validity",
  standard_concept: "standard concept",
  vocabulary_id: "vocabulary",
};

export default function ConceptFilterSummary({
  filters,
  onClear,
}: Readonly<ConceptFilterSummaryProps>) {
  return (
    <div style={{ display: "flex", justifyContent: "flex-end" }}>
      <div style={{ marginRight: "auto", textAlign: "left" }}>
        <div>Applied filters:</div>
        {Object.entries(filters).map(([key, values]) => {
          if (!values || values.length === 0) return null;
          const label = filterLabels[key as keyof typeof filterLabels];
          return (
            <div key={key}>
              {label}:{" "}
              {values.map((value) => (
                <Tag key={value.toString()}>{value.toString()}</Tag>
              ))}
            </div>
          );
        })}
      </div>
      <Button onClick={onClear}>clear filters</Button>
    </div>
  );
}
