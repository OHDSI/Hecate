import type { Key } from "react";
import type { TableProps } from "antd";
import { Link } from "react-router-dom";
import { ConceptRow } from "../@types/data-source";

type FilterField =
  | "concept_class_id"
  | "domain_id"
  | "invalid_reason"
  | "standard_concept"
  | "vocabulary_id";

interface FilterOption {
  text: string;
  value: string;
}

type FilterOptions = {
  conceptClass: FilterOption[];
  domain: FilterOption[];
  validity: FilterOption[];
  standard: FilterOption[];
  vocabulary: FilterOption[];
};

type FilteredInfo = Partial<Record<FilterField, Key[] | null>>;

interface ConceptTableColumnOptions {
  full: boolean;
  showFilters: boolean;
  filteredInfo: FilteredInfo;
  filterOptions: FilterOptions;
  renderCellValue: (row: ConceptRow, field: FilterField) => React.ReactNode;
}

export function buildConceptTableColumns({
  full,
  showFilters,
  filteredInfo,
  filterOptions,
  renderCellValue,
}: ConceptTableColumnOptions): NonNullable<TableProps<ConceptRow>["columns"]> {
  return [
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
      align: "left",
      minWidth: 105,
      render: (value, record) => (
        <div style={{ textAlign: "left" }}>
          {record.concept_id ? value : record.children?.length + " concepts"}
        </div>
      ),
    },
    {
      title: "code",
      dataIndex: "concept_code",
      key: "concept_code",
      minWidth: 120,
      responsive: full ? ["md"] : ["xxl"],
    },
    {
      title: "name",
      dataIndex: "concept_name",
      key: "concept_name",
      minWidth: 150,
      render: (value, row, index) => (
        <Link
          key={index + value}
          to={row.children ? "/" : `/concepts/${row.concept_id}`}
          style={{
            color: "#01452c",
            pointerEvents: row.children ? "none" : undefined,
          }}
        >
          {value}
        </Link>
      ),
      sorter: (a, b) => a.concept_name.localeCompare(b.concept_name),
    },
    {
      title: "class",
      dataIndex: "concept_class_id",
      key: "concept_class_id",
      filteredValue: showFilters ? filteredInfo.concept_class_id : undefined,
      filters: showFilters ? filterOptions.conceptClass : undefined,
      responsive: full ? ["md"] : ["xxl"],
      render: (_, row) => renderCellValue(row, "concept_class_id"),
      onFilter: showFilters
        ? (value, record) =>
            record.concept_class_id.toString().includes(value.toString())
        : undefined,
    },
    {
      title: "domain",
      dataIndex: "domain_id",
      key: "domain_id",
      filteredValue: showFilters ? filteredInfo.domain_id : undefined,
      filters: showFilters ? filterOptions.domain : undefined,
      responsive: full ? ["md"] : ["xxl"],
      onFilter: showFilters
        ? (value, record) => record.domain_id.includes(value as string)
        : undefined,
      render: (_, row) => renderCellValue(row, "domain_id"),
    },
    {
      title: "validity",
      dataIndex: "invalid_reason",
      key: "invalid_reason",
      filteredValue: showFilters ? filteredInfo.invalid_reason : undefined,
      filters: showFilters ? filterOptions.validity : undefined,
      responsive: full ? ["lg"] : ["xxl"],
      onFilter: showFilters
        ? (value, record) => record.invalid_reason.includes(value as string)
        : undefined,
      render: (_, row) => renderCellValue(row, "invalid_reason"),
    },
    {
      title: "concept",
      dataIndex: "standard_concept",
      key: "standard_concept",
      responsive: ["md"],
      filteredValue: showFilters ? filteredInfo.standard_concept : undefined,
      filters: showFilters ? filterOptions.standard : undefined,
      onFilter: showFilters
        ? (value, record) => record.standard_concept.includes(value as string)
        : undefined,
      render: (_, row) => renderCellValue(row, "standard_concept"),
    },
    {
      title: "vocabulary",
      dataIndex: "vocabulary_id",
      key: "vocabulary_id",
      responsive: ["sm"],
      filteredValue: showFilters ? filteredInfo.vocabulary_id : undefined,
      filters: showFilters ? filterOptions.vocabulary : undefined,
      onFilter: showFilters
        ? (value, record) => record.vocabulary_id.includes(value as string)
        : undefined,
      render: (_, row) => renderCellValue(row, "vocabulary_id"),
    },
    {
      title: "score",
      dataIndex: "score",
      key: "score",
      render: (value) => Math.round((value + Number.EPSILON) * 1000) / 1000,
      sorter: (a, b) => a.score - b.score,
      responsive: full ? ["xxl"] : ["md"],
    },
    {
      title: "records",
      dataIndex: "record_count",
      key: "record_count",
      render: (value: number | undefined) => value?.toLocaleString() ?? "",
      sorter: (a, b) => (a.record_count ?? 0) - (b.record_count ?? 0),
      responsive: full ? ["md"] : ["xxl"],
    },
  ];
}
