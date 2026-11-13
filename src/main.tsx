import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import App from "./App";
import NewTab from "./pages/NewTab";
import ErrorBoundary from "./components/ErrorBoundary";
import MenuPopup from "./pages/MenuPopup";
import Palette from "./pages/Palette";
import Toast from "./pages/Toast";
import Blocked from "./pages/Blocked";
import Vault from "./pages/Vault";
import { dbgLogFrontend } from "./api";

// Kabuk/iç sayfalardan gelen hataları sessizce yutmayıp rust.log'a düşür (tanı).
if (typeof window !== "undefined") {
  window.addEventListener("error", (e) => {
    dbgLogFrontend(`[window.error] ${e.message} @ ${e.filename}:${e.lineno}`).catch(() => {});
  });
  window.addEventListener("unhandledrejection", (e) => {
    dbgLogFrontend(`[unhandledrejection] ${String(e.reason)}`).catch(() => {});
  });
}
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
  toast: <Toast />,
  vault: <Vault />,
  blocked: <Blocked />,
  palette: <Palette />,
};

const pageToRender = internalPages[route];

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {pageToRender ?? <ErrorBoundary><App /></ErrorBoundary>}
  </React.StrictMode>
);
