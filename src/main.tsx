import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import DataProvider from "./store/DataContext";
import { ToastProvider } from "./hooks/ToastProvider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ToastProvider>
      <DataProvider>
        <App />
      </DataProvider>
    </ToastProvider>
  </React.StrictMode>,
);
