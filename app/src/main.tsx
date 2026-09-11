import React from "react";
import ReactDOM from "react-dom/client";
import Settings from "./Settings";
import { follow } from "./theme";
import "./index.css";

follow();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Settings />
  </React.StrictMode>,
);
