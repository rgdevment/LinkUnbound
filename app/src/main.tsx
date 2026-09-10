import { getCurrentWindow } from "@tauri-apps/api/window";
import React from "react";
import ReactDOM from "react-dom/client";
import Picker from "./Picker";
import Settings from "./Settings";
import "./index.css";

/// One bundle, two windows: which one this is decides what gets mounted.
const Root = getCurrentWindow().label === "settings" ? Settings : Picker;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
