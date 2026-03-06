import hecateLogo from "../assets/hecate-logo.png";
import pantheonLogo from "../assets/pantheon.svg";
import React, { useState } from "react";
import { Header } from "antd/es/layout/layout";
import { Dropdown, MenuProps, Modal } from "antd";
import SearchOutlinedIcon from "@mui/icons-material/SearchOutlined";
import {
  InfoOutlined,
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
        <div
          style={{
            display: "flex",
            alignItems: "center",
            marginRight: "auto",
            padding: "0 1em",
            fontSize: 12,
            color: "black",
            opacity: 0.85,
            lineHeight: 1.3,
          }}
        >
          Hecate is currently being updated and may not work as smoothly as you
          have come to expect.
          <br />
          Everything should be back to normal before Monday 9 March 00:00 CET.
        </div>
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
            <br /> It uses embeddings and cosine similarity to find search
            results and aims to provide a user friendly search experience that
            is intuitive and silky smooth.
            <br />
            Suggestions to improve the experience of using Hecate are always
            welcome.
            <br />
            The current vocabulary is the 27-FEB-26 release.
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
          <a
            href={"https://hecate.pantheon-hds.com/openapi"}
            target={"_blank"}
            rel="noopener noreferrer"
          >
            here
          </a>
          <br />
          Source code is available on GitHub{" "}
          <a
            href={"https://github.com/OHDSI/Hecate"}
            target={"_blank"}
            rel="noopener noreferrer"
          >
            here
          </a>
          <br />
          This public instance is provided for the OHDSI community by the
          <a
            href={
              "https://www.erasmusmc.nl/en/research/departments/medical-informatics"
            }
            target={"_blank"}
            rel="noopener noreferrer"
          >
            {" "}
            Erasmus MC Department of Medical Informatics
          </a>
          <br />
          info@pantheon-hds.com
        </div>
      </Modal>
    </Header>
  );
}

export default HecateHeader;
