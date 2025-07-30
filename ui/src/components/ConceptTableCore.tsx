import { useCallback, useMemo, useState } from "react";
import { Button, Table, TableProps, Tag } from "antd";
import { Link } from "react-router-dom";
import { ConceptRow } from "../@types/data-source";
import "../App.css";

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
  hiddenColumns?: string[];
  initialFilters?: Filters;
}

type FilterField = keyof FilterState;

const ConceptTableCore: React.FC<ConceptTableCoreProps> = ({
  data,
  loading = false,
  full = true,
  showFilters = false,
  hiddenColumns = [],
  initialFilters,
}) => {
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [filteredInfo, setFilteredInfo] = useState<Filters>(initialFilters || {});

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
          row[field] && concepts.push(row[field][0]);
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

  const columns: TableProps<ConceptRow>["columns"] = useMemo(
    () =>
      [
        {
          title: "",
          dataIndex: "",
          key: "concept_id",
          width: 1,
        },
        {
          title: "id",
          dataIndex: "concept_id",
          key: "concept_id",
          align: "left" as const,
          minWidth: 105,
          render: (value: any, record: ConceptRow) => (
            <div style={{ textAlign: "left" }}>
              {record.concept_id
                ? value
                : record.children?.length + " concepts"}
            </div>
          ),
        },
        {
          title: "code",
          dataIndex: "concept_code",
          key: "concept_code",
          minWidth: 120,
          responsive: full ? ["md" as const] : ["xxl" as const],
        },
        {
          title: "name",
          dataIndex: "concept_name",
          key: "concept_name",
          minWidth: 150,
          render: (value: string, row: ConceptRow, index: number) => {
            if (row.children) {
              return (
                <Link
                  key={index + value}
                  to="/"
                  style={{ color: "#01452c", pointerEvents: "none" }}
                >
                  {value}
                </Link>
              );
            } else {
              return (
                <Link
                  key={index + value}
                  to={`/concepts/${row.concept_id}`}
                  style={{ color: "#01452c" }}
                >
                  {value}
                </Link>
              );
            }
          },
          sorter: (a: ConceptRow, b: ConceptRow) =>
            a.concept_name.localeCompare(b.concept_name),
        },
        {
          title: "class",
          dataIndex: "concept_class_id",
          key: "concept_class_id",
          filteredValue: showFilters
            ? filteredInfo.concept_class_id
            : undefined,
          filters: showFilters ? filterOptions.conceptClass : undefined,
          responsive: full ? ["md" as const] : ["xxl" as const],
          render: (_: any, row: ConceptRow) =>
            renderCellValue(row, "concept_class_id"),
          onFilter: showFilters
            ? (value: any, record: ConceptRow) =>
                record["concept_class_id"].toString().includes(value.toString())
            : undefined,
        },
        {
          title: "domain",
          dataIndex: "domain_id",
          key: "domain_id",
          filteredValue: showFilters ? filteredInfo.domain_id : undefined,
          filters: showFilters ? filterOptions.domain : undefined,
          responsive: full ? ["md" as const] : ["xxl" as const],
          onFilter: showFilters
            ? (value: any, record: ConceptRow) =>
                record.domain_id.includes(value as string)
            : undefined,
          render: (_: any, row: ConceptRow) =>
            renderCellValue(row, "domain_id"),
        },
        {
          title: "validity",
          dataIndex: "invalid_reason",
          key: "invalid_reason",
          filteredValue: showFilters ? filteredInfo.invalid_reason : undefined,
          filters: showFilters ? filterOptions.validity : undefined,
          responsive: full ? ["lg" as const] : ["xxl" as const],
          onFilter: showFilters
            ? (value: any, record: ConceptRow) =>
                record.invalid_reason.includes(value as string)
            : undefined,
          render: (_: any, row: ConceptRow) =>
            renderCellValue(row, "invalid_reason"),
        },
        {
          title: "concept",
          dataIndex: "standard_concept",
          key: "standard_concept",
          responsive: ["md" as const],
          filteredValue: showFilters
            ? filteredInfo.standard_concept
            : undefined,
          filters: showFilters ? filterOptions.standard : undefined,
          onFilter: showFilters
            ? (value: any, record: ConceptRow) =>
                record.standard_concept.includes(value as string)
            : undefined,
          render: (_: any, row: ConceptRow) =>
            renderCellValue(row, "standard_concept"),
        },
        {
          title: "vocabulary",
          dataIndex: "vocabulary_id",
          key: "vocabulary_id",
          responsive: ["sm" as const],
          filteredValue: showFilters ? filteredInfo.vocabulary_id : undefined,
          filters: showFilters ? filterOptions.vocabulary : undefined,
          onFilter: showFilters
            ? (value: any, record: ConceptRow) =>
                record.vocabulary_id.includes(value as string)
            : undefined,
          render: (_: any, row: ConceptRow) =>
            renderCellValue(row, "vocabulary_id"),
        },
        {
          title: "score",
          dataIndex: "score",
          key: "score",
          render: (value: number) =>
            Math.round((value + Number.EPSILON) * 1000) / 1000,
          sorter: (a: ConceptRow, b: ConceptRow) => a.score - b.score,
          responsive: full ? ["md" as const] : ["xxl" as const],
        },
      ].filter((column) => !hiddenColumns.includes(column.key as string)),
    [
      full,
      showFilters,
      filteredInfo,
      filterOptions,
      renderCellValue,
      hiddenColumns,
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
