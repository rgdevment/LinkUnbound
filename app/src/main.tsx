import React from "react";
import ReactDOM from "react-dom/client";
import Settings from "./Settings";
import { follow } from "./theme";
import Welcome from "./Welcome";
import "./index.css";

follow();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {window.location.hash === "#welcome" ? <Welcome /> : <Settings />}
  </React.StrictMode>,
);
