import { useState } from "react";
import { Content } from "antd/es/layout/layout";
import { Button, Input, Typography } from "antd";

const { Title } = Typography;
import hecateLogo from "../assets/hecate-logo.png";
import { analyzeConceptSet, AnalysisResult } from "../service/conceptSets";
import HecateHeader from "./Header";
import ConceptRecommendationsComponent from "./ConceptRecommendations";

const { TextArea } = Input;

interface ConceptSetInputProps {
  conceptSetText: string;
  setConceptSetText: (text: string) => void;
  isAnalyzing: boolean;
  handleAnalyze: () => void;
}

const ConceptSetInput: React.FC<ConceptSetInputProps> = ({
  conceptSetText,
  setConceptSetText,
  isAnalyzing,
  handleAnalyze,
}) => {
  return (
    <>
      <div
        style={{
          marginBottom: "1em",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <Title level={5} style={{ margin: 0 }}>
          Concept Set
        </Title>
      </div>

      <TextArea
        value={conceptSetText}
        onChange={(e) => setConceptSetText(e.target.value)}
        placeholder="Paste concept set here for suggestions..."
        style={{
          marginBottom: "1em",
          fontSize: "11px",
          fontFamily: "monospace",
          width: "100%",
          height: "450px",
          resize: "none",
        }}
      />

      <div style={{ textAlign: "right", marginBottom: "1em" }}>
        <Button
          type="primary"
          size="large"
          loading={isAnalyzing}
          onClick={handleAnalyze}
          disabled={!conceptSetText.trim()}
        >
          Analyze Concept Set
        </Button>
      </div>
    </>
  );
};

function ConceptSets() {
  const [conceptSetText, setConceptSetText] = useState<string>("");
  const [isAnalyzing, setIsAnalyzing] = useState<boolean>(false);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResult | null>(
    null,
  );

  const handleAnalyze = async () => {
    setIsAnalyzing(true);
    setAnalysisResult(null);

    try {
      const result = await analyzeConceptSet(conceptSetText);
      setAnalysisResult(result);
    } catch (error) {
      console.error("Error analyzing concept set:", error);
      // Create error result for display
      setAnalysisResult({
        valid: false,
        errors: [`Network error: ${error}`],
        warnings: [],
      });
    } finally {
      setIsAnalyzing(false);
    }
  };

  return (
    <div>
      <HecateHeader />
      <Content>
        <div
          style={{
            paddingTop: "1em",
            paddingLeft: "2em",
            paddingRight: "2em",
          }}
        >
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginBottom: "2em",
            }}
          ></div>
          {analysisResult ? (
            <div>
              <div
                style={{
                  display: "flex",
                  gap: "2em",
                  alignItems: "flex-start",
                  marginBottom: "2em",
                }}
              >
                {/* Recommendations on the left */}
                <div style={{ flex: 1 }}>
                  {analysisResult.recommendations && (
                    <ConceptRecommendationsComponent
                      recommendations={analysisResult.recommendations}
                    />
                  )}
                </div>

                {/* Input field on the right */}
                <div style={{ flex: 1 }}>
                  <ConceptSetInput
                    conceptSetText={conceptSetText}
                    setConceptSetText={setConceptSetText}
                    isAnalyzing={isAnalyzing}
                    handleAnalyze={handleAnalyze}
                  />

                  {/* Concept Summary below button */}
                  {analysisResult.concept_summary && (
                    <div>
                      <div
                        style={{
                          marginBottom: "1em",
                          display: "flex",
                          justifyContent: "space-between",
                          alignItems: "center",
                        }}
                      >
                        <Title level={5} style={{ margin: 0 }}>
                          Summary
                        </Title>
                      </div>

                      <div
                        style={{
                          marginBottom: "1em",
                          padding: "1em",
                          border: "1px solid #d9d9d9",
                          borderRadius: "6px",
                          backgroundColor: "white",
                          textAlign: "left",
                        }}
                      >
                        <div
                          style={{
                            marginTop: "0.5em",
                            fontFamily: "monospace",
                            fontSize: "12px",
                          }}
                        >
                          <div>
                            • Included concepts:{" "}
                            {
                              analysisResult.concept_summary
                                .included_concepts_count
                            }
                          </div>
                          <div>
                            • Included descendants:{" "}
                            {
                              analysisResult.concept_summary
                                .included_descendants_count
                            }
                          </div>
                          <div>
                            • Included mapped:{" "}
                            {
                              analysisResult.concept_summary
                                .included_mapped_count
                            }
                          </div>
                          <div>
                            • Excluded concepts:{" "}
                            {
                              analysisResult.concept_summary
                                .excluded_concepts_count
                            }
                          </div>
                          <div>
                            • Excluded descendants:{" "}
                            {
                              analysisResult.concept_summary
                                .excluded_descendants_count
                            }
                          </div>
                          <div>
                            • Excluded mapped:{" "}
                            {
                              analysisResult.concept_summary
                                .excluded_mapped_count
                            }
                          </div>
                          <div
                            style={{ marginTop: "0.5em", fontWeight: "bold" }}
                          >
                            • Total included:{" "}
                            {analysisResult.concept_summary.total_included}
                          </div>
                          <div style={{ fontWeight: "bold" }}>
                            • Total excluded:{" "}
                            {analysisResult.concept_summary.total_excluded}
                          </div>
                        </div>
                      </div>
                    </div>
                  )}

                  {analysisResult.errors.length > 0 && (
                    <div
                      style={{
                        marginBottom: "1em",
                        padding: "1em",
                        border: "1px solid #ff4d4f",
                        borderRadius: "6px",
                        backgroundColor: "#fff2f0",
                      }}
                    >
                      <strong style={{ color: "#ff4d4f" }}>Errors:</strong>
                      <ul style={{ margin: "0.5em 0", paddingLeft: "2em" }}>
                        {analysisResult.errors.map((error, index) => (
                          <li key={index} style={{ color: "#ff4d4f" }}>
                            {error}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}

                  {analysisResult.warnings.length > 0 && (
                    <div
                      style={{
                        marginBottom: "1em",
                        padding: "1em",
                        border: "1px solid #faad14",
                        borderRadius: "6px",
                        backgroundColor: "#fffbe6",
                      }}
                    >
                      <strong style={{ color: "#faad14" }}>Warnings:</strong>
                      <ul style={{ margin: "0.5em 0", paddingLeft: "2em" }}>
                        {analysisResult.warnings.map((warning, index) => (
                          <li key={index} style={{ color: "#faad14" }}>
                            {warning}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              </div>
            </div>
          ) : (
            <div>
              <div
                style={{
                  display: "flex",
                  gap: "2em",
                  alignItems: "flex-start",
                  marginBottom: "2em",
                }}
              >
                {/* Logo on the left side */}
                <div
                  style={{
                    flex: 1,
                    display: "flex",
                    justifyContent: "center",
                    alignItems: "center",
                  }}
                >
                  {!analysisResult && (
                    <img
                      src={hecateLogo}
                      alt="Hecate logo"
                      style={{
                        height: "40vh",
                        opacity: 0.3,
                        mixBlendMode: "multiply",
                        maxWidth: "100%",
                        objectFit: "cover",
                      }}
                    />
                  )}
                </div>

                {/* Input field on the right */}
                <div style={{ flex: 1 }}>
                  <ConceptSetInput
                    conceptSetText={conceptSetText}
                    setConceptSetText={setConceptSetText}
                    isAnalyzing={isAnalyzing}
                    handleAnalyze={handleAnalyze}
                  />
                </div>
              </div>
            </div>
          )}
        </div>
      </Content>
    </div>
  );
}

export default ConceptSets;
