import { useState } from "react";
import "./App.css";
import { AutoComplete, Input } from "antd";
import { Content } from "antd/es/layout/layout";
import { autocomplete } from "./service/search.tsx";
import ConceptTable from "./components/ConceptTable.tsx";
import hecateLogo from "./assets/hecate-logo.png";
import HecateHeader from "./components/Header.tsx";

const dd = {
  width: "80%",
  margin: "2em",
  marginBottom: "0.5em",
};

function App() {
  const [searchTerm, setSearchTerm] = useState<string>("");
  const [suggestions, setSuggestions] = useState<
    { key: string; label: string; value: string }[]
  >([]);

  function doAutocomplete(q: string) {
    if (q && q.length > 2) {
      autocomplete(q).then((r) => {
        setSuggestions([]);
        const opts: { key: string; label: string; value: string }[] = r.map(
          (o, idx) => {
            return { key: o + idx, label: o, value: o };
          },
        );
        // Add current string if not already present
        if (!r.includes(q.toLowerCase())) {
          opts.unshift({ key: q, label: q, value: q });
        } else {
          opts.push({ key: q, label: q, value: q });
        }
        setSuggestions(opts);
      });
    } else {
      setSuggestions([{ key: q, label: q, value: q }]);
    }
  }

  return (
    <div>
      <HecateHeader />

      <Content>
        <div
          style={{
            width: "100vw",
            alignItems: "center",
            justifyContent: "center",
            marginBottom: "2em",
          }}
        >
          <AutoComplete
            classNames={{
              popup: {
                root: "certain-category-search-dropdown",
              },
            }}
            style={dd}
            onChange={doAutocomplete}
            onSelect={setSearchTerm}
            options={suggestions}
            defaultActiveFirstOption={true}
          >
            <Input.Search size="large" placeholder="Search for concepts" />
          </AutoComplete>
        </div>
        <div
          style={{
            paddingLeft: "5%",
            marginRight: "5%",
            paddingBottom: "2em",
            paddingTop: "1em",
          }}
        >
          {searchTerm ? (
            <ConceptTable searchTerm={searchTerm} full={true} />
          ) : (
            <img
              src={hecateLogo}
              alt="Hecate logo"
              style={{
                height: "50vh",
                opacity: 0.3,
                mixBlendMode: "multiply",
                maxWidth: "100%",
                objectFit: "cover",
              }}
            />
          )}
        </div>
      </Content>
    </div>
  );
}

export default App;
