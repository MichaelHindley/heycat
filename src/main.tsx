/* v8 ignore file -- @preserve */
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { initLogger } from "./lib/logger";

// Initialize logging (attaches Rust logs to browser console)
initLogger();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
