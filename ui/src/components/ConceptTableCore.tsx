import { useCallback, useMemo, useState } from "react";
import { Button, Table, TableProps, Tag } from "antd";
import { ConceptRow } from "../@types/data-source";
import "../App.css";
import { buildConceptTableColumns } from "./conceptTableColumns";

type OnChange = NonNullable<TableProps<ConceptRow>["onChange"]>;
type Filters = Parameters<OnChange>[1];

interface FilterOption {
  text: string;
  value: string;
}

interface FilterState {
  concept_class_id?: string[];
  domain_id?: string[];
  invalid_reason?: string[];
  standard_concept?: string[];
  vocabulary_id?: string[];
}

interface ConceptTableCoreProps {
  data: ConceptRow[];
  loading?: boolean;
  full?: boolean;
  showFilters?: boolean;
  hiddenColumns?: readonly string[];
  initialFilters?: Filters;
}

type FilterField = keyof FilterState;
const EMPTY_HIDDEN_COLUMNS: readonly string[] = [];

const ConceptTableCore: React.FC<ConceptTableCoreProps> = ({
  data,
  loading = false,
  full = true,
  showFilters = false,
  hiddenColumns = EMPTY_HIDDEN_COLUMNS,
  initialFilters,
}) => {
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [filteredInfo, setFilteredInfo] = useState<Filters>(
    initialFilters || {},
  );

  const handleChange: OnChange = useCallback((_, filters) => {
    setFilteredInfo(filters);
  }, []);

  const clearFilters = useCallback(() => {
    setFilteredInfo({
      concept_class_id: null,
      domain_id: null,
      invalid_reason: null,
      standard_concept: null,
      vocabulary_id: null,
    });
  }, []);

  const renderTagsForField = useCallback(
    (filtered: ConceptRow[], field: FilterField) => {
      const values = [
        ...new Set(filtered.map((r) => (r[field] ? r[field][0] : ""))),
      ].filter(Boolean);
      return (
        <>
          {values.map((tag) => (
            <Tag key={tag}>{tag}</Tag>
          ))}
        </>
      );
    },
    [],
  );

  const renderCellValue = useCallback(
    (row: ConceptRow, field: FilterField) => {
      if (row.children) {
        return renderTagsForField(row.children, field);
      }
      return row[field] ? row[field][0] : "";
    },
    [renderTagsForField],
  );

  // Generate filter options from data
  const filterOptions = useMemo(() => {
    const createCountForFilter = (concepts: string[]): FilterOption[] => {
      const counts = concepts.reduce(
        (acc, concept) => {
          acc[concept] = (acc[concept] || 0) + 1;
          return acc;
        },
        {} as Record<string, number>,
      );

      return Object.entries(counts).map(([key, count]) => ({
        text: `${key} (${count})`,
        value: key,
      }));
    };

    const getFilterSelector = (field: FilterField): FilterOption[] => {
      const concepts: string[] = [];
      data.forEach((row) => {
        if (row.children) {
          row.children.forEach(
            (child) => child[field] && concepts.push(child[field][0]),
          );
        } else {
          if (row[field]) concepts.push(row[field][0]);
        }
      });
      concepts.sort((a, b) => a.localeCompare(b));
      return createCountForFilter(concepts);
    };

    return {
      conceptClass: getFilterSelector("concept_class_id"),
      domain: getFilterSelector("domain_id"),
      validity: getFilterSelector("invalid_reason"),
      standard: getFilterSelector("standard_concept"),
      vocabulary: getFilterSelector("vocabulary_id"),
    };
  }, [data]);

  const hiddenColumnSet = useMemo(
    () => new Set(hiddenColumns),
    [hiddenColumns],
  );

  const columns = useMemo(
    () =>
      buildConceptTableColumns({
        full,
        showFilters,
        filteredInfo,
        filterOptions,
        renderCellValue,
      }).filter(
        (column) => !column || !hiddenColumnSet.has(column.key as string),
      ),
    [
      full,
      showFilters,
      filteredInfo,
      filterOptions,
      renderCellValue,
      hiddenColumnSet,
    ],
  );

  const hasActiveFilters =
    showFilters &&
    Object.values(filteredInfo).some((filter) => filter && filter.length > 0);

  const renderFilterSection = () => {
    if (!hasActiveFilters) return null;

    const filterLabels = {
      concept_class_id: "class",
      domain_id: "domain",
      invalid_reason: "validity",
      standard_concept: "standard concept",
      vocabulary_id: "vocabulary",
    };

    return (
      <div style={{ display: "flex", justifyContent: "flex-end" }}>
        <div style={{ marginRight: "auto", textAlign: "left" }}>
          <div>Applied filters:</div>
          {Object.entries(filteredInfo).map(([key, values]) => {
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
        <Button onClick={clearFilters}>clear filters</Button>
      </div>
    );
  };

  return (
    <div>
      {renderFilterSection()}
      <Table
        rowKey={(record) => record.concept_id + record.concept_name}
        style={{ paddingTop: "1em", fontSize: "8px" }}
        columns={columns}
        onChange={showFilters ? handleChange : undefined}
        dataSource={data}
        expandable={{
          childrenColumnName: "",
          indentSize: 5,
          expandedRowClassName: "expand-row",
        }}
        pagination={{
          current: currentPage,
          onChange: (page) => {
            setCurrentPage(page);
          },
          defaultCurrent: 1,
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
        loading={loading}
      />
    </div>
  );
};

export default ConceptTableCore;
