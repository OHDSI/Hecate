import { Content } from "antd/es/layout/layout";
import { Row, Table, TableProps, Tabs, TabsProps, Col, Card } from "antd";
import { useEffect, useMemo, useState } from "react";
import { Concept } from "../@types/data-source";
import { getConceptById, getConceptDefinition } from "../service/concepts.tsx";
import { useParams } from "react-router-dom";
import {
  expandInvalidReasonAbbreviation,
  expandStandardConceptAbbreviation,
} from "../service/search.tsx";
import ConceptTable from "./ConceptTable.tsx";
import RelatedConceptsView from "./RelatedConceptsView.tsx";
import HecateHeader from "./Header.tsx";
import ConceptHierarchyTable from "./ConceptHierarchyTable.tsx";

const columns: TableProps<
  { name: string; value: string } | { name: string; value: number }
>["columns"] = [
  {
    dataIndex: "name",
    key: "name",
    width: 150,
  },
  {
    dataIndex: "value",
    key: "value",
    width: 250,
  },
];

function ConceptView() {
  const [concept, setConcept] = useState<Concept | undefined>(undefined);
  const params = useParams();
  const conceptId = useMemo(
    () => Number.parseInt(params.id ?? "0", 10),
    [params.id],
  );
  const [definition, setDefinition] = useState<string>("");
  const [data, setData] = useState<
    ({ name: string; value: string } | { name: string; value: number })[]
  >([]);

  useEffect(() => {
    if (params.id) {
      let cancelled = false;

      getConceptById(conceptId).then((resp) => {
        if (cancelled) return;
        const c = resp[0];
        setConcept(c);
        const d = [
          { name: "domain", value: c.domain_id },
          { name: "class", value: c.concept_class_id },
          { name: "vocabulary", value: c.vocabulary_id },
          { name: "id", value: c.concept_id },
          { name: "code", value: c.concept_code },
          {
            name: "concept",
            value: expandStandardConceptAbbreviation(c.standard_concept),
          },
          {
            name: "validity",
            value: expandInvalidReasonAbbreviation(c.invalid_reason),
          },
          { name: "valid start", value: c.valid_start_date },
          { name: "valid end", value: c.valid_end_date },
          { name: "record count", value: c.record_count.toLocaleString() },
        ];
        setData(d);
      });
      getConceptDefinition(conceptId).then((resp) => {
        if (!cancelled) setDefinition(resp);
      });

      return () => {
        cancelled = true;
      };
    }
  }, [conceptId, params.id]);

  const tabItems: TabsProps["items"] = [
    {
      key: "1",
      label: "Related Concepts",
      children: concept ? (
        <RelatedConceptsView conceptId={conceptId} phoebe={false} />
      ) : (
        <div>Loading ... </div>
      ),
    },
    {
      key: "2",
      label: "Similar",
      children: concept ? (
        <ConceptTable
          key={concept.concept_name}
          searchTerm={concept.concept_name}
          full={false}
        />
      ) : (
        <div>Loading ... </div>
      ),
    },
    {
      key: "3",
      label: "PHOEBE",
      children: concept ? (
        <RelatedConceptsView conceptId={conceptId} phoebe={true} />
      ) : (
        <div>Loading ... </div>
      ),
    },
    {
      key: "4",
      label: "Descendents",
      children: concept ? (
        <ConceptHierarchyTable
          key={conceptId}
          conceptId={conceptId}
          full={true}
        />
      ) : (
        <div>Loading ... </div>
      ),
      // disabled: true,
    },
  ];

  const firstTabItems: TabsProps["items"] = [
    {
      key: "1",
      label: "Details",
      children: (
        <Table
          size={"small"}
          style={{ paddingTop: "1em", fontSize: "8px" }}
          // showHeader={false}
          columns={columns}
          dataSource={data}
          pagination={{ hideOnSinglePage: true, pageSize: 25 }}
        />
      ),
    },
    {
      key: "2",
      label: "Definition",
      children: (
        <div style={{ paddingTop: "1em", textAlign: "left" }}>
          <Card>{definition}</Card>
        </div>
      ),
      disabled: false,
    },
  ];

  return (
    <div>
      <HecateHeader />
      <Content style={{ paddingLeft: "5%", paddingRight: "5%" }}>
        <Row>
          <h2
            style={{
              marginRight: "auto",
            }}
          >
            {concept?.concept_name}
          </h2>
        </Row>
        <Row>
          <Col span={5}>
            <Tabs defaultActiveKey={"1"} items={firstTabItems} />
          </Col>
          <Col span={19}>
            <Tabs
              style={{ paddingLeft: "5%" }}
              defaultActiveKey={"1"}
              items={tabItems}
            />
          </Col>
        </Row>
      </Content>
    </div>
  );
}

export default ConceptView;
