import { ConceptRow } from "../@types/data-source";
import { useCallback, useEffect, useMemo, useState } from "react";
import "../App.css";
import { Button, notification, Table, TableProps, Tag } from "antd";
import { search } from "../service/search.tsx";
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

interface ConceptTableProps {
  searchTerm: string;
  full: boolean;
}

type FilterField = keyof FilterState;

const useConceptFilters = () => {
  const [filteredInfo, setFilteredInfo] = useState<Filters>({});
  const [currentFilters, setCurrentFilters] = useState<FilterState>({});
  const [filterOptions, setFilterOptions] = useState<{
    conceptClass: FilterOption[];
    domain: FilterOption[];
    validity: FilterOption[];
    standard: FilterOption[];
    vocabulary: FilterOption[];
  }>({
    conceptClass: [],
    domain: [],
    validity: [],
    standard: [],
    vocabulary: [],
  });

  const clearFilters = useCallback(() => {
    setFilteredInfo({
      concept_class_id: null,
      domain_id: null,
      invalid_reason: null,
      standard_concept: null,
      vocabulary_id: null,
    });
    setCurrentFilters({});
  }, []);

  const handleChange: OnChange = useCallback((_, filters) => {
    setFilteredInfo(filters);
    setCurrentFilters({
      concept_class_id: filters.concept_class_id?.toString().split(","),
      domain_id: filters.domain_id?.toString().split(","),
      invalid_reason: filters.invalid_reason?.toString().split(","),
      standard_concept: filters.standard_concept?.toString().split(","),
      vocabulary_id: filters.vocabulary_id?.toString().split(","),
    });
  }, []);

  return {
    filteredInfo,
    currentFilters,
    filterOptions,
    setFilterOptions,
    clearFilters,
    handleChange,
  };
};

export default function ConceptTable(props: Readonly<ConceptTableProps>) {
  const { searchTerm, full } = props;
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [loading, setLoading] = useState<boolean>(false);
  const [selected, setSelected] = useState<ConceptRow[]>([]);
  const [originalData, setOriginalData] = useState<ConceptRow[]>([]);

  const {
    filteredInfo,
    currentFilters,
    filterOptions,
    setFilterOptions,
    clearFilters,
    handleChange,
  } = useConceptFilters();

  const resetToOriginalData = useCallback(() => {
    setSelected(originalData);
  }, [originalData]);

  const modifiedClearFilters = useCallback(() => {
    clearFilters();
    resetToOriginalData();
  }, [clearFilters, resetToOriginalData]);

  // If due to a result of the filter there is only one child we don't want to expand
  const applyFiltersToChildren = useCallback(
    (children: ConceptRow[], filters: FilterState): ConceptRow[] => {
      const conceptClassIds = new Set(filters.concept_class_id);
      const domainIds = new Set(filters.domain_id);
      const invalidReasons = new Set(filters.invalid_reason);
      const standardConcepts = new Set(filters.standard_concept);
      const vocabularyIds = new Set(filters.vocabulary_id);

      return children.filter((child) => {
        if (
          conceptClassIds.size &&
          !conceptClassIds.has(child.concept_class_id[0])
        ) {
          return false;
        }
        if (domainIds.size && !domainIds.has(child.domain_id[0])) {
          return false;
        }
        if (
          invalidReasons.size &&
          !invalidReasons.has(child.invalid_reason[0])
        ) {
          return false;
        }
        if (
          standardConcepts.size &&
          !standardConcepts.has(child.standard_concept[0])
        ) {
          return false;
        }
        if (vocabularyIds.size && !vocabularyIds.has(child.vocabulary_id[0])) {
          return false;
        }
        return true;
      });
    },
    [],
  );

  const filterData = useCallback(() => {
    const filteredSelection: ConceptRow[] = originalData.map((row) => {
      if (row.children && row.children.length > 1) {
        const acceptedChildren = applyFiltersToChildren(
          row.children,
          currentFilters,
        );
        if (acceptedChildren.length === 1) {
          return { ...acceptedChildren[0] };
        } else {
          return { ...row, children: acceptedChildren };
        }
      }
      return { ...row };
    });
    setSelected(filteredSelection);
  }, [originalData, currentFilters, applyFiltersToChildren]);

  const applyFilterForConceptsWithChildren = useCallback(
    (children: ConceptRow[]) => {
      if (!filteredInfo) {
        return children;
      }
      return applyFiltersToChildren(children, currentFilters);
    },
    [filteredInfo, currentFilters, applyFiltersToChildren],
  );

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
        const filtered = applyFilterForConceptsWithChildren(row.children);
        if (filtered.length === 1) {
          return filtered[0][field] ? filtered[0][field][0] : "";
        }
        return renderTagsForField(filtered, field);
      }
      return row[field];
    },
    [applyFilterForConceptsWithChildren, renderTagsForField],
  );

  const columns = useMemo(
    () =>
      buildConceptTableColumns({
        full,
        showFilters: true,
        filteredInfo,
        filterOptions,
        renderCellValue,
      }),
    [full, filteredInfo, filterOptions, renderCellValue],
  );

  const createCountForFilter = useCallback(
    (concepts: string[]): FilterOption[] => {
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
    },
    [],
  );

  const getFilterSelector = useCallback(
    (field: FilterField, rows: ConceptRow[]): FilterOption[] => {
      const concepts: string[] = [];
      rows.forEach((row) => {
        if (row.children) {
          row.children.forEach((child) => concepts.push(child[field][0]));
        } else {
          concepts.push(row[field][0]);
        }
      });
      concepts.sort((a, b) => a.localeCompare(b));
      return createCountForFilter(concepts);
    },
    [createCountForFilter],
  );

  const openNotification = useCallback(() => {
    notification.error({
      title: "Oops",
      message:
        "Something went wrong, get in touch report issues to info@pantheon-hds.com",
      placement: "topRight",
    });
  }, []);

  const doSearch = useCallback(
    async (q: string) => {
      setLoading(true);
      try {
        const results = await search(q);
        setFilterOptions({
          conceptClass: getFilterSelector("concept_class_id", results),
          domain: getFilterSelector("domain_id", results),
          validity: getFilterSelector("invalid_reason", results),
          standard: getFilterSelector("standard_concept", results),
          vocabulary: getFilterSelector("vocabulary_id", results),
        });
        setSelected(results);
        setOriginalData(results);
      } catch {
        openNotification();
      } finally {
        setLoading(false);
      }
    },
    [getFilterSelector, openNotification, setFilterOptions],
  );

  useEffect(() => {
    void doSearch(searchTerm);
  }, [searchTerm, doSearch]);

  useEffect(() => {
    filterData();
  }, [filterData]);

  const hasActiveFilters = Object.values(filteredInfo).some(
    (filter) => filter && filter.length > 0,
  );

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
        <Button onClick={modifiedClearFilters}>clear filters</Button>
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
        onChange={handleChange}
        dataSource={selected}
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
}
