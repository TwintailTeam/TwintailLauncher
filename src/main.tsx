import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { DialogProvider } from "./context/DialogContext";
import DialogOverlay from "./components/layout/DialogOverlay";
import DialogEventHandler from "./components/layout/DialogEventHandler";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <DialogProvider>
      <App />
      <DialogOverlay />
      <DialogEventHandler />
    </DialogProvider>
  </React.StrictMode>,
);
