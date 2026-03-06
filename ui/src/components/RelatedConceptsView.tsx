import { useCallback, useEffect, useState } from "react";
import { RelatedConcept } from "../@types/data-source";
import { getPhoebeConcepts, getRelatedConcepts } from "../service/concepts.tsx";
import { Button, Table, TableProps, Tag } from "antd";
import { Link } from "react-router-dom";

type OnChange = NonNullable<TableProps<RelatedConcept>["onChange"]>;
type Filters = Parameters<OnChange>[1];

export default function RelatedConceptsView(props: {
  conceptId: number;
  phoebe: boolean;
}) {
  const { conceptId, phoebe } = props;
  const [concepts, setConcepts] = useState<RelatedConcept[]>([]);
  const [vocabFilter, setVocabFilter] = useState<
    { text: string; value: string }[]
  >([]);
  const [relationShipIdFilter, setRelationShipIdFilter] = useState<
    { text: string; value: string }[]
  >([]);

  const [filteredInfo, setFilteredInfo] = useState<Filters>({});

  const createCountForFilter = useCallback((concepts: string[]) => {
    const counts = concepts.reduce<Record<string, number>>((p, c) => {
      p[c] = (p[c] || 0) + 1;
      return p;
    }, {});
    return Object.keys(counts).map((k) => ({
      text: k + " (" + counts[k] + ")",
      value: k,
    }));
  }, []);

  const getFilterSelector = useCallback(
    (field: "vocabulary_id" | "relationship_id", row: RelatedConcept[]) => {
      const concepts: string[] = [];
      row.forEach((r) => {
        concepts.push(r[field]);
      });
      concepts.sort((a, b) => a.localeCompare(b));
      return createCountForFilter(concepts);
    },
    [createCountForFilter],
  );

  useEffect(() => {
    if (!phoebe) {
      getRelatedConcepts(conceptId).then((resp) => {
        const filtered = resp.filter((rc) => rc.concept_id !== conceptId);
        setConcepts(filtered);
        setVocabFilter(getFilterSelector("vocabulary_id", resp));
        setRelationShipIdFilter(getFilterSelector("relationship_id", resp));
      });
    } else {
      getPhoebeConcepts(conceptId).then((resp) => {
        const filtered = resp.filter((rc) => rc.concept_id !== conceptId);
        setConcepts(filtered);
        setVocabFilter(getFilterSelector("vocabulary_id", resp));
        setRelationShipIdFilter(getFilterSelector("relationship_id", resp));
      });
    }
  }, [conceptId, getFilterSelector, phoebe]);

  const clearFilters = () => {
    setFilteredInfo({
      vocabulary_id: null,
      relationship_id: null,
    });
  };

  const handleChange: OnChange = (_pagination, filters) => {
    setFilteredInfo(filters);
  };

  const columns: TableProps<RelatedConcept>["columns"] = [
    {
      title: "relationship",
      dataIndex: "relationship_id",
      key: "relationship_id",
      align: "left",
      filteredValue: filteredInfo.relationship_id,
      filters: relationShipIdFilter,
      onFilter: (value, record) =>
        record.relationship_id.includes(value as string),
      minWidth: 120,
    },
    {
      title: "id",
      dataIndex: "concept_id",
      key: "concept_id",
      align: "left",
      minWidth: 120,
      render: (value) => <div style={{ textAlign: "left" }}>{value}</div>,
    },
    {
      title: "name",
      dataIndex: "concept_name",
      key: "concept_name",
      minWidth: 160,
      render: (value, row, index) => {
        return (
          <Link
            key={index + value}
            to={`/concepts/${row.concept_id}`}
            style={{ color: "#01452c" }}
          >
            {value}
          </Link>
        );
      },
      sorter: (a, b) => a.concept_name.localeCompare(b.concept_name),
    },
    {
      title: "vocabulary",
      dataIndex: "vocabulary_id",
      key: "vocabulary_id",
      filteredValue: filteredInfo.vocabulary_id,
      filters: vocabFilter,
      onFilter: (value, record) =>
        record.vocabulary_id.includes(value as string),
    },
    {
      title: "records",
      dataIndex: "record_count",
      key: "record_count",
      render: (value: number | undefined) => value?.toLocaleString() ?? "",
      sorter: (a, b) => (a.record_count ?? 0) - (b.record_count ?? 0),
      responsive: ["md"],
    },
  ];

  return (
    <div style={{ paddingBottom: "4em" }}>
      {(filteredInfo.relationship_id || filteredInfo.vocabulary_id) && (
        <div
          style={{
            display: "flex",
            justifyContent: "flex-end",
          }}
        >
          <div style={{ marginRight: "auto", textAlign: "left" }}>
            <div>Applied filters:</div>
            {filteredInfo.relationship_id && (
              <div>
                relationship:{" "}
                {filteredInfo.relationship_id.map((d) => (
                  <Tag key={d.toString()}>{d.toString()}</Tag>
                ))}
              </div>
            )}
            {filteredInfo.vocabulary_id && (
              <div>
                vocabulary:{" "}
                {filteredInfo.vocabulary_id.map((d) => (
                  <Tag key={d.toString()}>{d.toString()}</Tag>
                ))}
              </div>
            )}
          </div>
          <Button onClick={clearFilters}>clear filters</Button>
        </div>
      )}
      <Table
        rowKey={(record) => record.concept_id + record.relationship_id}
        style={{ paddingTop: "1em", fontSize: "8px" }}
        columns={columns}
        onChange={handleChange}
        dataSource={concepts}
        expandable={{
          childrenColumnName: "",
          indentSize: 5,
          expandedRowClassName: "expand-row",
        }}
        pagination={{
          defaultCurrent: 1,
          defaultPageSize: 20,
          hideOnSinglePage: true,
          showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
        }}
      />
    </div>
  );
}
