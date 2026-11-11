// QuantumBrowser — içerik iç sayfaları (quantum://*) sarmalayıcıları
// Ana kabuk dışında, çocuk webview'da bağımsız çalışır; snapshot'ı çeker.
import { useCallback, useEffect, useState } from "react";
import type { Snapshot } from "../types";
import { applyRootTheme } from "../utils/theme";
import * as api from "../api";
import {
  BookmarksPanel,
  DownloadsPanel,
  HistoryPanel,
  ProfilesPanel,
  SettingsPanel,
} from "../components/Panels";

function useInternalPage() {
  const [snap, setSnap] = useState<Snapshot | null>(null);

  const refresh = useCallback(() => {
    api.getSnapshot().then(setSnap).catch(() => {});
  }, []);

  useEffect(() => {
    refresh();
    const un = api.onTabsChanged(refresh);
    return () => { un.then((f) => f()); };
  }, [refresh]);

  // Hesap, tema, accent ve block_insecure anlık senkronu
  useEffect(() => {
    if (!snap) return;
    applyRootTheme(snap.theme || "light", snap.accent || "mor");
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => applyRootTheme(snap.theme || "light", snap.accent || "mor");
    mq.addEventListener?.("change", onChange);
    return () => mq.removeEventListener?.("change", onChange);
  }, [snap?.theme, snap?.accent]);

  const closeSelf = useCallback(() => {
    api.getSnapshot().then((s) => {
      if (s.active_id) api.closeTab(s.active_id);
    });
  }, []);

  return { snap, refresh, closeSelf };
}

export function SettingsPage() {
  const { snap, refresh, closeSelf } = useInternalPage();
  if (!snap) return null;
  return (
    <SettingsPanel
      snapshot={snap}
      onClose={closeSelf}
      onSetTheme={(t) => api.setTheme(t).then(refresh)}
      onSetEngine={(e) => api.setSearchEngine(e).then(refresh)}
      onToggleBlocker={(v) => api.setBlockerEnabled(v)}
      onClearData={() => api.clearBrowsingData().then(refresh)}
      onSetSessionRestore={(v) => api.setSessionRestore(v).then(refresh)}
      onSetAccent={(a) => api.setAccent(a).then(refresh)}
      onSetBlockInsecure={(v) => api.setBlockInsecure(v).then(refresh)}
      onOpenVault={() => api.newTab("quantum://vault")}
    />
  );
}

export function HistoryPage() {
  const { closeSelf } = useInternalPage();
  return <HistoryPanel onClose={closeSelf} onNavigate={(url) => api.navigate(url)} />;
}

export function BookmarksPage() {
  const { closeSelf } = useInternalPage();
  return <BookmarksPanel onClose={closeSelf} onNavigate={(url) => api.navigate(url)} />;
}

export function DownloadsPage() {
  const { closeSelf } = useInternalPage();
  return <DownloadsPanel onClose={closeSelf} />;
}

export function ProfilesPage() {
  const { snap, refresh, closeSelf } = useInternalPage();
  if (!snap) return null;
  return (
    <ProfilesPanel
      snapshot={snap}
      onClose={closeSelf}
      onRefresh={refresh}
      onSwitch={(id) => api.switchProfile(id).then(refresh)}
    />
  );
}
