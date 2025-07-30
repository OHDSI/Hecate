import hecateLogo from "../assets/hecate-logo.png";
import pantheonLogo from "../assets/pantheon.svg";
import React, { useState } from "react";
import { Header } from "antd/es/layout/layout";
import { Dropdown, MenuProps, Modal } from "antd";
import SearchOutlinedIcon from "@mui/icons-material/SearchOutlined";
import {
  HubOutlined,
  InfoOutlined,
  LoginOutlined,
  MenuOutlined,
  WorkspacesOutlined,
} from "@mui/icons-material";
import { Link } from "react-router-dom";

const headerStyle: React.CSSProperties = {
  height: 64,
  padding: 0,
  backgroundImage: "linear-gradient(to left, white 16%, #01452c 84%)",
};

function HecateHeader() {
  const [modalOpen, setModalOpen] = useState<boolean>(false);

  const items: MenuProps["items"] = [
    {
      key: "1",
      label: (
        <Link
          key={23}
          to="/"
          style={{ fontSize: "large", fontFamily: "GFS Neohellenic" }}
        >
          Search
        </Link>
      ),
      icon: <SearchOutlinedIcon style={{ fontSize: "large" }} />,
    },
    {
      key: "2",
      label: (
        <Link
          to="/concept-sets"
          style={{ fontSize: "large", fontFamily: "GFS Neohellenic" }}
        >
          Concept Sets
        </Link>
      ),
      icon: <WorkspacesOutlined style={{ fontSize: "large" }} />,
    },
    {
      key: "3",
      label: (
        <div style={{ fontSize: "large", fontFamily: "GFS Neohellenic" }}>
          Map
        </div>
      ),
      icon: <HubOutlined style={{ fontSize: "large" }} />,
      disabled: true,
    },
    {
      key: "4",
      label: (
        <div style={{ fontSize: "large", fontFamily: "GFS Neohellenic" }}>
          Login
        </div>
      ),
      icon: <LoginOutlined style={{ fontSize: "large" }} />,
      disabled: true,
    },
    {
      key: "5",
      label: (
        <div
          onClick={() => setModalOpen(true)}
          style={{ fontSize: "large", fontFamily: "GFS Neohellenic" }}
        >
          About
        </div>
      ),
      icon: <InfoOutlined style={{ fontSize: "large" }} />,
    },
  ];

  return (
    <Header style={headerStyle}>
      <div
        style={{
          display: "flex",
          justifyContent: "flex-end",
          height: "100%",
        }}
      >
        <Dropdown menu={{ items }}>
          <div
            style={{
              marginRight: "auto",
              paddingLeft: "1%",
              fontWeight: "500",
              fontSize: 38,
              fontFamily: "GFS Neohellenic",
              color: "#F3F7FC",
            }}
          >
            <MenuOutlined style={{ marginRight: "0.5em" }} />

            <Link key={23} to="/" style={{ color: "#F3F7FC" }}>
              Hecate
            </Link>
          </div>
        </Dropdown>
        <img
          onClick={() => setModalOpen(true)}
          src={hecateLogo}
          alt="Hecate logo"
          style={{
            mixBlendMode: "multiply",
            height: "100%",
            opacity: 0.8,
            paddingRight: "0.3em",
            cursor: "pointer",
          }}
        />
      </div>
      <Modal
        style={{ fontFamily: "GFS Neohellenic", color: "#01452c" }}
        title="About"
        centered
        open={modalOpen}
        closable={true}
        footer={null}
        onCancel={() => setModalOpen(false)}
      >
        <div
          style={{
            display: "flex",
            height: "2em",
            justifyContent: "flex-start",
          }}
        >
          <div>
            Hecate is a semantic search engine for the OHDSI vocabulary.
            <br /> It uses LLM embeddings and cosine similarity to provide
            relevant search results and aims to provide a user friendly search
            experience that is intuitive and silky smooth.
            <br />
            Suggestions to improve the experience of using Hecate are always
            welcome.
            <br />
            The current vocabulary is the 30-AUG-24 release.
          </div>
        </div>
        <img
          src={pantheonLogo}
          alt={"Pantheon logo"}
          style={{
            transform: "scale(0.7)",
          }}
        />
        <div>
          OpenAPI Specifications are available{" "}
          <a href={"https://hecate.pantheon-hds.com/openapi"} target={"_blank"}>
            here
            <br />
          </a>
          Source code is available on GitHub{" "}
          <a href={"https://github.com/OHDSI/Hecate"} target={"_blank"}>
            here
            <br />
          </a>
          info@pantheon-hds.com
        </div>
      </Modal>
    </Header>
  );
}

export default HecateHeader;
