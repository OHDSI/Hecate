import { useCallback, useEffect, useState } from "react";
import { Content } from "antd/es/layout/layout";
import { Button, Table, Tag, Card, TableProps } from "antd";
import type { ColumnsType } from "antd/es/table";
import HecateHeader from "./Header.tsx";

type OnChange = NonNullable<TableProps<MappingResult>["onChange"]>;
type Filters = Parameters<OnChange>[1];

interface FilterOption {
  text: string;
  value: string;
}

interface FilterState {
  target_concept_class_id?: string[];
  target_vocabulary_id?: string[];
  mapping_type?: string[];
}

type FilterField = keyof FilterState;

interface MappingResult {
  source: {
    concept_id: number | null;
    concept_name: string;
    concept_class_id: string | null;
    vocabulary_id: string | null;
    country: string | null;
  };
  mapping_date: string;
  maps_to: {
    concept_id: number;
    concept_name: string;
    concept_class_id: string;
    vocabulary_id: string;
    type: string;
    inferred: string | null;
    validated_by: string | null;
  } | null;
  decomposition: {
    ingredients: Array<{
      input: {
        name: string;
        name_inferred: boolean;
        strength: string | null;
        strength_inferred: boolean;
      };
      maps_to: {
        concept_id: number;
        concept_name: string;
        concept_class_id: string;
        vocabulary_id: string;
        type: string;
        rationale: string | null;
        validated_by: string | null;
      };
    }>;
    dose_form: {
      name: string;
      rationale: string | null;
    } | null;
    brand: {
      name: string;
      maps_to: {
        concept_id: number;
        concept_name: string;
        concept_class_id: string;
        vocabulary_id: string;
        type: string;
        rationale: string | null;
        validated_by: string | null;
      };
    } | null;
    supplier: {
      name: string;
      maps_to: {
        concept_id: number;
        concept_name: string;
        concept_class_id: string;
        vocabulary_id: string;
        type: string;
        rationale: string | null;
        validated_by: string | null;
      };
    } | null;
    quantity: string | null;
    rationale: string;
  };
}

const useMappingFilters = () => {
  const [filteredInfo, setFilteredInfo] = useState<Filters>({});
  const [currentFilters, setCurrentFilters] = useState<FilterState>({});
  const [filterOptions, setFilterOptions] = useState<{
    conceptClass: FilterOption[];
    vocabulary: FilterOption[];
    mappingType: FilterOption[];
  }>({
    conceptClass: [],
    vocabulary: [],
    mappingType: [],
  });

  const clearFilters = useCallback(() => {
    setFilteredInfo({
      target_concept_class_id: null,
      target_vocabulary_id: null,
      mapping_type: null,
    });
    setCurrentFilters({});
  }, []);

  const handleChange: OnChange = useCallback((_, filters) => {
    setFilteredInfo(filters);
    setCurrentFilters({
      target_concept_class_id: filters.target_concept_class_id as string[],
      target_vocabulary_id: filters.target_vocabulary_id as string[],
      mapping_type: filters.mapping_type as string[],
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

function MappingResults() {
  const [data, setData] = useState<MappingResult[]>([]);
  const [loading, setLoading] = useState(true);
  const [currentPage, setCurrentPage] = useState<number>(1);

  const {
    filteredInfo,
    filterOptions,
    setFilterOptions,
    clearFilters,
    handleChange,
  } = useMappingFilters();

  const createCountForFilter = useCallback(
    (items: string[]): FilterOption[] => {
      const counts = items.reduce(
        (acc, item) => {
          acc[item] = (acc[item] || 0) + 1;
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
    (field: FilterField, rows: MappingResult[]): FilterOption[] => {
      const items: string[] = [];
      rows.forEach((row) => {
        if (row.maps_to) {
          if (field === "target_concept_class_id") {
            items.push(row.maps_to.concept_class_id);
          } else if (field === "target_vocabulary_id") {
            items.push(row.maps_to.vocabulary_id);
          } else if (field === "mapping_type") {
            items.push(row.maps_to.type);
          }
        }
      });
      items.sort((a, b) => a.localeCompare(b));
      return createCountForFilter(items);
    },
    [createCountForFilter],
  );

  useEffect(() => {
    // fetch("http://localhost:8080/api/drug-mapping")
    fetch("/sample-mapping-results.json")
      .then((response) => response.json())
      .then((jsonData: MappingResult[]) => {
        setData(jsonData);
        setFilterOptions({
          conceptClass: getFilterSelector("target_concept_class_id", jsonData),
          vocabulary: getFilterSelector("target_vocabulary_id", jsonData),
          mappingType: getFilterSelector("mapping_type", jsonData),
        });
        setLoading(false);
      })
      .catch((error) => {
        console.error("Error loading mapping results:", error);
        setLoading(false);
      });
  }, [getFilterSelector, setFilterOptions]);

  const getTypeColor = (type: string) => {
    switch (type) {
      case "EXACT":
        return "default";
      case "NARROW":
        return "orange";
      case "BROAD":
        return "default";
      case "INCORRECT":
        return "red";
      default:
        return "default";
    }
  };

  const expandedRowRender = (record: MappingResult) => {
    return (
      <div style={{ padding: "16px", backgroundColor: "#fafafa" }}>
        {record.decomposition.rationale && (
          <Card size="small" style={{ marginBottom: "16px" }}>
            <div style={{ fontStyle: "italic" }}>
              {record.decomposition.rationale}
            </div>
          </Card>
        )}

        {record.decomposition.dose_form && (
          <div style={{ marginBottom: "16px" }}>
            <div></div>
            <Table
              dataSource={[
                {
                  ...record.decomposition.dose_form,
                  key: "dose-form",
                },
              ]}
              columns={[
                {
                  title: "dose form",
                  dataIndex: "name",
                  key: "dose_form_name",
                  width: "65%",
                  render: (value) => value || "Not specified",
                },
                {
                  title: "rationale",
                  dataIndex: "rationale",
                  key: "dose_form_rationale",
                  render: (value) =>
                    value ? (
                      <div style={{ fontStyle: "italic" }}>{value}</div>
                    ) : (
                      "Not specified"
                    ),
                },
              ]}
              pagination={false}
              size="small"
              rowKey="key"
              showHeader={true}
            />
          </div>
        )}

        {record.decomposition.quantity && (
          <div style={{ marginBottom: "16px" }}>
            <div></div>
            <Table
              dataSource={[
                {
                  quantity: record.decomposition.quantity,
                  key: "quantity",
                },
              ]}
              columns={[
                {
                  title: "quantity",
                  dataIndex: "quantity",
                  key: "quantity",
                  render: (value) => value || "Not specified",
                },
              ]}
              pagination={false}
              size="small"
              rowKey="key"
              showHeader={true}
            />
          </div>
        )}
        {record.decomposition.ingredients.length > 0 && (
          <div style={{ marginBottom: "16px" }}>
            <div></div>
            <Table
              dataSource={record.decomposition.ingredients.map(
                (ingredient, index) => ({
                  ...ingredient,
                  key: index,
                }),
              )}
              columns={[
                {
                  title: "ingredient",
                  dataIndex: ["input", "name"],
                  key: "ingredient_name",
                  width: "20%",
                },
                {
                  title: "strength",
                  dataIndex: ["input", "strength"],
                  key: "ingredient_strength",
                  width: "10%",
                  render: (value) => value || "Not specified",
                },
                {
                  title: "maps to",
                  dataIndex: ["maps_to", "concept_name"],
                  key: "ingredient_maps_to",
                  width: "25%",
                  render: (value, record) => {
                    const rec = record as unknown as { maps_to?: { concept_id: number } };
                    return rec.maps_to ? (
                      <a
                        href={`/concepts/${rec.maps_to.concept_id}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        style={{ color: "#01452c" }}
                      >
                        {value}
                      </a>
                    ) : (
                      "N/A"
                    );
                  },
                },
                {
                  title: "mapping type",
                  dataIndex: ["maps_to", "type"],
                  key: "ingredient_type",
                  width: "10%",
                  render: (type: string) => (
                    <Tag color={getTypeColor(type)}>{type}</Tag>
                  ),
                },
                {
                  title: "rationale",
                  dataIndex: ["maps_to", "rationale"],
                  key: "ingredient_rationale",
                  render: (value) =>
                    value ? (
                      <div style={{ fontStyle: "italic" }}>{value}</div>
                    ) : (
                      "Not specified"
                    ),
                },
              ]}
              pagination={false}
              size="small"
              rowKey="key"
            />
          </div>
        )}

        {record.decomposition.brand && (
          <div style={{ marginBottom: "16px" }}>
            <div></div>
            <Table
              dataSource={[
                {
                  ...record.decomposition.brand,
                  key: "brand",
                } as Record<string, unknown>,
              ]}
              columns={[
                {
                  title: "brand",
                  dataIndex: "name",
                  key: "brand_name",
                  width: "30%",
                },
                {
                  title: "maps to",
                  dataIndex: ["maps_to", "concept_name"],
                  key: "brand_maps_to",
                  width: "25%",
                  render: (value, record) => {
                    const rec = record as unknown as { maps_to?: { concept_id: number } };
                    return rec.maps_to ? (
                      <a
                        href={`/concepts/${rec.maps_to.concept_id}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        style={{ color: "#01452c" }}
                      >
                        {value}
                      </a>
                    ) : (
                      "N/A"
                    );
                  },
                },
                {
                  title: "mapping type",
                  dataIndex: ["maps_to", "type"],
                  key: "brand_type",
                  width: "10%",
                  render: (type: string) => (
                    <Tag color={getTypeColor(type)}>{type}</Tag>
                  ),
                },
                {
                  title: "rationale",
                  dataIndex: ["maps_to", "rationale"],
                  key: "brand_rationale",
                  render: (value) =>
                    value ? (
                      <div style={{ fontStyle: "italic" }}>{value}</div>
                    ) : (
                      "Not specified"
                    ),
                },
              ]}
              pagination={false}
              size="small"
              rowKey="key"
            />
          </div>
        )}
        {record.decomposition.supplier && (
          <div style={{ marginBottom: "16px" }}>
            <div></div>
            <Table
              dataSource={[
                {
                  ...record.decomposition.supplier,
                  key: "supplier",
                } as Record<string, unknown>,
              ]}
              columns={[
                {
                  title: "supplier",
                  dataIndex: "name",
                  key: "supplier_name",
                  width: "30%",
                },
                {
                  title: "maps to",
                  dataIndex: ["maps_to", "concept_name"],
                  key: "supplier_maps_to",
                  width: "25%",
                  render: (value, record) => {
                    const rec = record as unknown as { maps_to?: { concept_id: number } };
                    return rec.maps_to ? (
                      <a
                        href={`/concepts/${rec.maps_to.concept_id}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        style={{ color: "#01452c" }}
                      >
                        {value}
                      </a>
                    ) : (
                      "N/A"
                    );
                  },
                },
                {
                  title: "mapping type",
                  dataIndex: ["maps_to", "type"],
                  key: "supplier_type",
                  width: "10%",
                  render: (type: string) => (
                    <Tag color={getTypeColor(type)}>{type}</Tag>
                  ),
                },
                {
                  title: "rationale",
                  dataIndex: ["maps_to", "rationale"],
                  key: "supplier_rationale",
                  render: (value) =>
                    value ? (
                      <div style={{ fontStyle: "italic" }}>{value}</div>
                    ) : (
                      "Not specified"
                    ),
                },
              ]}
              pagination={false}
              size="small"
              rowKey="key"
            />
          </div>
        )}
      </div>
    );
  };

  const columns: ColumnsType<MappingResult> = [
    {
      title: "code",
      dataIndex: ["source", "concept_id"],
      key: "source_concept_id",
      render: (value) => value || "N/A",
      minWidth: 80,
      align: "left",
      sorter: (a, b) => {
        const aValue = a.source.concept_id?.toString() || "";
        const bValue = b.source.concept_id?.toString() || "";
        return aValue.localeCompare(bValue);
      },
    },
    {
      title: "source input",
      dataIndex: ["source", "concept_name"],
      key: "source_concept_name",
      minWidth: 150,
      sorter: (a, b) =>
        a.source.concept_name.localeCompare(b.source.concept_name),
    },
    {
      title: "maps to",
      dataIndex: ["maps_to", "concept_id"],
      key: "maps_to_concept_id",
      minWidth: 105,
      align: "left",
      render: (_value, record) => record.maps_to?.concept_id || "N/A",
      sorter: (a, b) =>
        (a.maps_to?.concept_id || 0) - (b.maps_to?.concept_id || 0),
    },
    {
      title: "concept name",
      dataIndex: ["maps_to", "concept_name"],
      key: "maps_to_concept_name",
      minWidth: 150,
      render: (value, record) =>
        record.maps_to ? (
          <a
            href={`/concepts/${record.maps_to.concept_id}`}
            target="_blank"
            rel="noopener noreferrer"
            style={{ color: "#01452c" }}
          >
            {value}
          </a>
        ) : (
          "No mapping"
        ),
      sorter: (a, b) =>
        (a.maps_to?.concept_name || "").localeCompare(
          b.maps_to?.concept_name || "",
        ),
    },
    {
      title: "class",
      dataIndex: ["maps_to", "concept_class_id"],
      key: "target_concept_class_id",
      responsive: ["md"],
      minWidth: 140,
      render: (_value, record) => record.maps_to?.concept_class_id || "N/A",
      filteredValue: filteredInfo.target_concept_class_id,
      filters: filterOptions.conceptClass,
      onFilter: (value, record) =>
        record.maps_to?.concept_class_id === value.toString(),
    },
    {
      title: "vocabulary",
      dataIndex: ["maps_to", "vocabulary_id"],
      key: "target_vocabulary_id",
      minWidth: 140,
      responsive: ["lg"],
      render: (_value, record) => record.maps_to?.vocabulary_id || "N/A",
      filteredValue: filteredInfo.target_vocabulary_id,
      filters: filterOptions.vocabulary,
      onFilter: (value, record) =>
        record.maps_to?.vocabulary_id === value.toString(),
    },
    {
      title: "type",
      dataIndex: ["maps_to", "type"],
      key: "mapping_type",
      render: (type: string, record) =>
        record.maps_to ? (
          <div style={{ textAlign: "center" }}>
            <Tag color={getTypeColor(type)}>{type}</Tag>
          </div>
        ) : (
          "N/A"
        ),
      width: 90,
      align: "left",
      filteredValue: filteredInfo.mapping_type,
      filters: filterOptions.mappingType,
      onFilter: (value, record) => record.maps_to?.type === value.toString(),
    },
    {
      title: "date",
      dataIndex: "mapping_date",
      key: "mapping_date",
      align: "left",
      width: 110,
      sorter: (a, b) =>
        new Date(a.mapping_date).getTime() - new Date(b.mapping_date).getTime(),
    },
  ];

  return (
    <div>
      <HecateHeader />
      <Content>
        <div
          style={{
            paddingLeft: "5%",
            marginRight: "5%",
            paddingBottom: "2em",
            paddingTop: "3em",
          }}
        >
          {/*<div style={{ marginBottom: "24px" }}>*/}
          {/*  <div style={{ display: "flex", gap: "12px", alignItems: "flex-end" }}>*/}
          {/*    <div style={{ flex: 0.5 }}>*/}
          {/*      <div style={{ marginBottom: "8px", fontWeight: "500", textAlign: "left" }}>code</div>*/}
          {/*      <Input*/}
          {/*        placeholder="Enter code..."*/}
          {/*        value={drugCode}*/}
          {/*        onChange={(e) => setDrugCode(e.target.value)}*/}
          {/*        size="large"*/}
          {/*        style={{*/}
          {/*          borderRadius: "8px",*/}
          {/*          fontSize: "16px",*/}
          {/*        }}*/}
          {/*      />*/}
          {/*    </div>*/}
          {/*    <div style={{ flex: 2 }}>*/}
          {/*      <div style={{ marginBottom: "8px", fontWeight: "500", textAlign: "left" }}>details</div>*/}
          {/*      <Input*/}
          {/*        placeholder="Enter details..."*/}
          {/*        value={drugName}*/}
          {/*        onChange={(e) => setDrugName(e.target.value)}*/}
          {/*        onPressEnter={handleInputSubmit}*/}
          {/*        size="large"*/}
          {/*        style={{*/}
          {/*          borderRadius: "8px",*/}
          {/*          fontSize: "16px",*/}
          {/*        }}*/}
          {/*      />*/}
          {/*    </div>*/}
          {/*    <Button*/}
          {/*      type="primary"*/}
          {/*      size="large"*/}
          {/*      onClick={handleInputSubmit}*/}
          {/*      style={{*/}
          {/*        borderRadius: "8px",*/}
          {/*        height: "40px",*/}
          {/*        paddingLeft: "24px",*/}
          {/*        paddingRight: "24px",*/}
          {/*      }}*/}
          {/*    >*/}
          {/*      map*/}
          {/*    </Button>*/}
          {/*  </div>*/}
          {/*</div>*/}
          {(() => {
            const hasActiveFilters = Object.values(filteredInfo).some(
              (filter) => filter && filter.length > 0,
            );

            if (!hasActiveFilters) return null;

            const filterLabels = {
              target_concept_class_id: "class",
              target_vocabulary_id: "vocabulary",
              mapping_type: "mapping type",
            };

            return (
              <div
                style={{
                  display: "flex",
                  justifyContent: "flex-end",
                  marginBottom: "16px",
                }}
              >
                <div style={{ marginRight: "auto", textAlign: "left" }}>
                  <div>Applied filters:</div>
                  {Object.entries(filteredInfo).map(([key, values]) => {
                    if (!values || values.length === 0) return null;
                    const label =
                      filterLabels[key as keyof typeof filterLabels];
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
          })()}

          <Table
            style={{ marginTop: "1em" }}
            columns={columns}
            dataSource={data}
            loading={loading}
            onChange={handleChange}
            expandable={{
              expandedRowRender,
              expandRowByClick: false,
            }}
            rowKey={(record, index) =>
              `${record.maps_to?.concept_id || "null"}-${index}`
            }
            pagination={{
              current: currentPage,
              onChange: (page) => {
                setCurrentPage(page);
              },
              pageSize: 20,
              showQuickJumper: true,
              showTotal: (total, range) =>
                `${range[0]}-${range[1]} of ${total} items`,
            }}
            scroll={{ x: 1200 }}
            size="middle"
          />
        </div>
      </Content>
    </div>
  );
}

export default MappingResults;
