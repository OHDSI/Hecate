import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.css";
import { ConfigProvider, Layout } from "antd";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import ConceptView from "./components/ConceptView.tsx";
import ConceptSets from "./components/ConceptSets.tsx";

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
  },
  {
    path: "/search",
    element: <App />,
  },
  {
    path: "/concepts/:id",
    element: <ConceptView />,
  },
  {
    path: "/concept-sets",
    element: <ConceptSets />,
  },
]);

const layoutStyle = {
  borderRadius: 0,
  backgroundColor: "#F3F7FC",
  width: "100vw",
  minHeight: "100vh",
  maxHeight: "100%",
};

ReactDOM.createRoot(document.getElementById("root")!).render(
  // <React.StrictMode>
  <ConfigProvider
    theme={{
      token: {
        // Seed Token
        colorPrimary: "#01452c",
      },
    }}
  >
    <Layout style={layoutStyle}>
      <RouterProvider router={router} />
    </Layout>
  </ConfigProvider>,
  // </React.StrictMode>,
);
