import { getCurrentWindow } from "@tauri-apps/api/window";
import React from "react";
import ReactDOM from "react-dom/client";
import Notice from "./Notice";
import Picker from "./Picker";
import Settings from "./Settings";
import { follow } from "./theme";
import "./index.css";

/// One bundle, three windows: which one this is decides what gets mounted.
const ROOTS = { settings: Settings, notice: Notice } as const;
const label = getCurrentWindow().label as keyof typeof ROOTS;
const Root = ROOTS[label] ?? Picker;

follow();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
