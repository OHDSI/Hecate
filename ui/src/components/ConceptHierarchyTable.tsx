import { ConceptExpandRow } from "../@types/data-source";
import { Link } from "react-router-dom";
import { useEffect, useState } from "react";
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
  const [currentPage, setCurrentPage] = useState<number>(0);
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
      minWidth: 120,
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
      title: "distance",
      dataIndex: "level",
      key: "level",
      responsive: full ? ["md"] : ["xxl"],
      render: (_, row) => {
        return row.level - 1;
      },
    },
  ];
  const [selected, setSelected] = useState<ConceptExpandRow[]>([]);

  const openNotification = () => {
    notification.error({
      message: `Oops`,
      description:
        "Something went wrong, get in touch report issues to info@pantheon-hds.com",
      placement: "topRight",
    });
  };

  useEffect(() => {
    doSearch(conceptId);
  }, [conceptId]);

  function doSearch(conceptId: number) {
    setLoading(true);
    getConceptExpand(conceptId)
      .then((r) => {
        setLoading(false);
        setCurrentPage(1);
        setSelected(r);
      })
      .catch(() => {
        openNotification();
        setLoading(false);
      });
  }

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
