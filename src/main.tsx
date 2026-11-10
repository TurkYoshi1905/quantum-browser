import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import App from "./App";
import NewTab from "./pages/NewTab";
import ErrorBoundary from "./components/ErrorBoundary";

// Çocuk webview'lar "index.html#/newtab" ile yüklenir → sadece NewTab sayfasını çiz.
// Kabuk penceresi hash'siz yüklenir → tam QuantumBrowser kabuğu.
const isNewTabPage = window.location.hash.startsWith("#/newtab");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {isNewTabPage ? <NewTab /> : <ErrorBoundary><App /></ErrorBoundary>}
  </React.StrictMode>
);
