import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { CombinedStore } from "./store";
import { ToastProvider } from "./hooks/ToastProvider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ToastProvider>
      <CombinedStore>
        <App />
      </CombinedStore>
    </ToastProvider>
  </React.StrictMode>,
);
