/* v8 ignore file -- @preserve */
import React from "react";
import ReactDOM from "react-dom/client";
import { CatOverlay } from "./components/CatOverlay";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <CatOverlay />
  </React.StrictMode>,
);
