import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import App from "./App";
import NewTab from "./pages/NewTab";
import ErrorBoundary from "./components/ErrorBoundary";
import MenuPopup from "./pages/MenuPopup";
import Blocked from "./pages/Blocked";
import Vault from "./pages/Vault";
import {
  SettingsPage,
  HistoryPage,
  BookmarksPage,
  DownloadsPage,
  ProfilesPage,
} from "./pages/Internal";

// Çocuk webview'lar "index.html#/<rota>" ile yüklenir → ilgili iç sayfa çizilir.
// Kabuk penceresi hash'siz yüklenir → tam QuantumBrowser kabuğu.
const route = window.location.hash.replace(/^#\/?/, "").split(/[?#]/)[0];

const internalPages: Record<string, React.ReactNode> = {
  newtab: <NewTab />,
  settings: <SettingsPage />,
  history: <HistoryPage />,
  bookmarks: <BookmarksPage />,
  downloads: <DownloadsPage />,
  profiles: <ProfilesPage />,
  popover: <MenuPopup />,
  vault: <Vault />,
  blocked: <Blocked />,
};

const pageToRender = internalPages[route];

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {pageToRender ?? <ErrorBoundary><App /></ErrorBoundary>}
  </React.StrictMode>
);
