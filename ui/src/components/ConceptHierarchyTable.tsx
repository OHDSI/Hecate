import { ConceptExpandRow } from "../@types/data-source";
import { Link } from "react-router-dom";
import { useCallback, useEffect, useState } from "react";
import "../App.css";
import { notification, Table, TableProps } from "antd";
import { getConceptExpand } from "../service/concepts.tsx";

export default function ConceptHierarchyTable(
  props: Readonly<{
    conceptId: number;
    full: boolean;
  }>,
) {
  const { conceptId, full } = props;
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [loading, setLoading] = useState<boolean>(false);

  const columns: TableProps<ConceptExpandRow>["columns"] = [
    {
      title: "id",
      dataIndex: "concept_id",
      key: "concept_id",
      align: "left",
      minWidth: 105,
    },
    {
      title: "code",
      dataIndex: "concept_code",
      key: "concept_code",
      minWidth: 110,
      responsive: full ? ["md"] : ["xxl"],
    },

    {
      title: "name",
      dataIndex: "concept_name",
      key: "concept_name",
      minWidth: 150,
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
      minWidth: 120,
    },
    {
      title: "distance",
      dataIndex: "level",
      key: "level",
      responsive: full ? ["md"] : ["xxl"],
      render: (_, row) => {
        return row.level - 1;
      },
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
  const [selected, setSelected] = useState<ConceptExpandRow[]>([]);

  const openNotification = useCallback(() => {
    notification.error({
      title: `Oops`,
      message:
        "Something went wrong, get in touch report issues to info@pantheon-hds.com",
      placement: "topRight",
    });
  }, []);

  const doSearch = useCallback(
    async (conceptId: number) => {
      setLoading(true);
      try {
        const results = await getConceptExpand(conceptId);
        setSelected(results);
      } catch {
        openNotification();
      } finally {
        setLoading(false);
      }
    },
    [openNotification],
  );

  useEffect(() => {
    void doSearch(conceptId);
  }, [conceptId, doSearch]);

  return (
    <Table
      rowKey={(record) => record.concept_id}
      style={{ paddingTop: "1em", fontSize: "8px" }}
      columns={columns}
      dataSource={selected}
      pagination={{
        current: currentPage,
        onChange: (page) => {
          setCurrentPage(page);
        },
        defaultCurrent: 1,
        pageSize: 30,
        showTotal: (total, range) => `${range[0]}-${range[1]} of ${total}`,
      }}
      loading={loading}
    />
  );
}
