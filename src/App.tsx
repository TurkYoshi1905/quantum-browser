import { useCallback, useEffect, useRef, useState } from "react";
import { Check } from "lucide-react";
import * as api from "./api";
import type { Snapshot } from "./types";
import TitleBar from "./components/TitleBar";
import Sidebar, { type Panel } from "./components/Sidebar";
import AddressBar from "./components/AddressBar";
import {
  BookmarksPanel,
  DownloadsPanel,
  HistoryPanel,
  ProfilesPanel,
  SettingsPanel,
} from "./components/Panels";

export default function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [panel, setPanel] = useState<Panel | null>(null);
  const [bookmarked, setBookmarked] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const omniboxRef = useRef<HTMLInputElement>(null);
  const snapshotRef = useRef<Snapshot | null>(null);
  snapshotRef.current = snapshot;

  const refresh = useCallback(() => {
    api.getSnapshot().then(setSnapshot).catch(() => {});
  }, []);

  // Veri yenileme + blok sayacı dinleyicisi
  useEffect(() => {
    refresh();
    const un1 = api.onTabsChanged(refresh);
    const un2 = api.onBlockerCount(({ id, count }) => {
      setSnapshot((prev) =>
        prev
          ? { ...prev, tabs: prev.tabs.map((t) => (t.id === id ? { ...t, blocked_count: count } : t)) }
          : prev
      );
    });
    return () => {
      un1.then((f) => f());
      un2.then((f) => f());
    };
  }, [refresh]);

  // İndirme tamamlanınca bildirim baloncuğu
  const seenDownloads = useRef<Set<number>>(new Set());
  useEffect(() => {
    const un = api.onDownloadsChanged(async () => {
      try {
        const items = await api.listDownloads();
        for (const d of items) {
          if (d.state === "completed" && !seenDownloads.current.has(d.id)) {
            seenDownloads.current.add(d.id);
            setToast(d.file_name);
            window.setTimeout(() => setToast(null), 4200);
          }
        }
      } catch {
        // sessiz geç
      }
    });
    return () => {
      un.then((f) => f());
    };
  }, []);

  // Tema uygula (sistem = OS tercihi)
  const theme = snapshot?.theme;
  useEffect(() => {
    if (!theme) return;
    const apply = () => {
      const dark =
        theme === "dark" || (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
      document.documentElement.dataset.theme = dark ? "dark" : "light";
    };
    apply();
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    mq.addEventListener?.("change", apply);
    return () => mq.removeEventListener?.("change", apply);
  }, [theme]);

  // Panel açıldığında çocuk webview'ları gizle (panel öne gelsin)
  useEffect(() => {
    api.setOverlayOpen(panel !== null).catch(() => {});
  }, [panel]);

  const activeTab = snapshot?.tabs.find((t) => t.id === snapshot.active_id) ?? null;
  const activeUrl = activeTab?.url ?? "";
  const isActiveNewTab = activeTab?.is_newtab ?? true;
  const isActiveIncognito = activeTab?.incognito ?? false;

  // Yer imi yıldızı durumu
  useEffect(() => {
    if (!activeUrl || isActiveNewTab) {
      setBookmarked(false);
      return;
    }
    api.isBookmarked(activeUrl).then(setBookmarked).catch(() => setBookmarked(false));
  }, [activeUrl, isActiveNewTab]);

  const closePanel = useCallback(() => setPanel(null), []);

  const cycleTab = useCallback(
    (direction: 1 | -1) => {
      const snap = snapshotRef.current;
      if (!snap || snap.tabs.length === 0) return;
      const idx = snap.tabs.findIndex((t) => t.id === snap.active_id);
      const next = (idx + direction + snap.tabs.length) % snap.tabs.length;
      api.activateTab(snap.tabs[next].id);
    },
    []
  );

  const activateNth = useCallback((n: number) => {
    const snap = snapshotRef.current;
    if (!snap) return;
    const target = n === 9 ? snap.tabs[snap.tabs.length - 1] : snap.tabs[n - 1];
    if (target) api.activateTab(target.id);
  }, []);

  // Klavye kısayolları (kabuk odaktayken; sayfa odaktayken v0.2'de WebView2
  // AcceleratorKeyPressed kancası devreye girecek)
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // F12 — geliştirici panelini aç/kapat (kabuğun dışında, sayfa bağlamında)
      if (e.key === "F12" || ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "i")) {
        e.preventDefault();
        api.toggleDevtools();
        return;
      }

      const mod = e.ctrlKey || e.metaKey;
      const alt = e.altKey;
      const shift = e.shiftKey;
      const key = e.key.toLowerCase();

      if (key === "escape") {
        setPanel(null);
        return;
      }
      if (key === "f5") {
        e.preventDefault();
        api.reload();
        return;
      }
      if (alt && key === "arrowleft") {
        e.preventDefault();
        api.goBack();
        return;
      }
      if (alt && key === "arrowright") {
        e.preventDefault();
        api.goForward();
        return;
      }
      if (!mod) return;

      if (key === "tab") {
        e.preventDefault();
        cycleTab(shift ? -1 : 1);
        return;
      }
      if (/^[1-9]$/.test(key)) {
        e.preventDefault();
        activateNth(parseInt(key, 10));
        return;
      }

      const run = (fn: () => void) => {
        e.preventDefault();
        fn();
      };
      if (key === "t" && !shift) return run(() => api.newTab());
      if (key === "n" && shift) return run(() => api.newIncognitoTab());
      if (key === "w") {
        const id = snapshotRef.current?.active_id;
        if (id) return run(() => api.closeTab(id));
        return;
      }
      if (key === "l" || key === "k") return run(() => omniboxRef.current?.focus());
      if (key === "r") return run(() => api.reload());
      if (key === "b" && !shift) return run(() => setPanel("bookmarks"));
      if (key === "j") return run(() => setPanel("downloads"));
      if (key === "," ) return run(() => setPanel("settings"));
      if (key === "y" && shift) return run(() => setPanel("history"));
      if (key === "delete" && shift) return run(() => setPanel("settings"));
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [activateNth, cycleTab]);

  const handleNavigate = useCallback((input: string) => {
    const trimmed = input.trim();
    if (trimmed) api.navigate(trimmed);
  }, []);

  const handlePanelNavigate = useCallback((url: string) => {
    setPanel(null);
    api.navigate(url);
  }, []);

  const handleToggleBookmark = useCallback(async () => {
    const snap = snapshotRef.current;
    const tab = snap?.tabs.find((t) => t.id === snap.active_id);
    if (!tab || tab.is_newtab || !tab.url) return;
    try {
      const now = await api.toggleBookmark(tab.url, tab.title || tab.url);
      setBookmarked(now);
    } catch {
      /* sessiz */
    }
  }, []);

  const handleToggleBlocker = useCallback(() => {
    const snap = snapshotRef.current;
    if (!snap) return;
    api.setBlockerEnabled(!snap.blocker_enabled);
  }, []);

  const handleSwitchProfile = useCallback(async (profileId: number) => {
    try {
      await api.switchProfile(profileId);
      setPanel(null);
      await api.newTab();
      refresh();
    } catch {
      /* sessiz */
    }
  }, [refresh]);

  const handleCreateGroupAndAssign = useCallback((tabId: string, name: string, color: string) => {
    api.createGroup(name, color).then((groupId) => api.assignGroup(tabId, groupId)).then(refresh);
  }, [refresh]);

  // ---------- Yükleniyor ekranı ----------
  if (!snapshot) {
    return (
      <div className="shell" style={{ gridTemplateRows: "1fr" }}>
        <div className="newtab" style={{ minHeight: "100vh" }}>
          <svg width="72" height="72" viewBox="0 0 32 32" fill="none" aria-hidden="true">
            <ellipse cx="16" cy="16" rx="12.5" ry="4.6" transform="rotate(-30 16 16)" stroke="#22d3ee" strokeWidth="2.2" strokeLinecap="round" />
            <ellipse cx="16" cy="16" rx="12.5" ry="4.6" transform="rotate(55 16 16)" stroke="#a78bfa" strokeWidth="2.2" strokeLinecap="round" />
            <circle cx="16" cy="16" r="4.4" fill="#67e8f9" />
          </svg>
          <div className="greeting" style={{ marginTop: 10 }}>QuantumBrowser yükleniyor…</div>
        </div>
      </div>
    );
  }

  const activeTitle = activeTab ? (activeTab.is_newtab ? "Yeni Sekme" : activeTab.title || activeTab.url) : "QuantumBrowser";

  return (
    <div className="shell">
      <TitleBar profileName={snapshot.profile_name} title={activeTitle} />

      <div className="main">
        <Sidebar
          snapshot={snapshot}
          onToggleSidebar={() => api.setSidebarCollapsed(!snapshot.sidebar_collapsed)}
          onNewTab={() => api.newTab()}
          onNewIncognito={() => api.newIncognitoTab()}
          onClose={(id) => api.closeTab(id)}
          onActivate={(id) => api.activateTab(id)}
          onDuplicate={(id) => api.duplicateTab(id)}
          onTogglePin={(id) => api.togglePin(id)}
          onCloseOthers={(id) => api.closeOthers(id)}
          onMoveTab={(fromId, toId) => api.moveTab(fromId, toId)}
          onToggleGroup={(groupId) => api.toggleGroup(groupId).then(refresh)}
          onAssignGroup={(tabId, groupId) => api.assignGroup(tabId, groupId).then(refresh)}
          onCreateGroupAndAssign={handleCreateGroupAndAssign}
          onOpenPanel={setPanel}
        />

        <div className="content">
          <AddressBar
            url={activeUrl}
            isIncognito={isActiveIncognito}
            isNewTab={isActiveNewTab}
            blockedCount={activeTab?.blocked_count ?? 0}
            bookmarked={bookmarked}
            blockerEnabled={snapshot.blocker_enabled}
            inputRef={omniboxRef}
            onNavigate={handleNavigate}
            onBack={api.goBack}
            onForward={api.goForward}
            onReload={api.reload}
            onToggleBookmark={handleToggleBookmark}
            onToggleBlocker={handleToggleBlocker}
            onNewTab={() => api.newTab()}
            onNewIncognito={() => api.newIncognitoTab()}
            onOpenPanel={setPanel}
          />

          <div className="page-area">
            <div className="page-empty" aria-hidden="true">
              {!panel && <span style={{ opacity: 0.35 }}>QuantumBrowser</span>}
            </div>

            {panel === "history" && <HistoryPanel onClose={closePanel} onNavigate={handlePanelNavigate} />}
            {panel === "bookmarks" && <BookmarksPanel onClose={closePanel} onNavigate={handlePanelNavigate} />}
            {panel === "downloads" && <DownloadsPanel onClose={closePanel} />}
            {panel === "settings" && (
              <SettingsPanel
                snapshot={snapshot}
                onClose={closePanel}
                onSetTheme={(t) => api.setTheme(t).then(refresh)}
                onSetEngine={(e) => api.setSearchEngine(e).then(refresh)}
                onToggleBlocker={(v) => api.setBlockerEnabled(v)}
                onClearData={() =>
                  api.clearBrowsingData().then(() => {
                    setToast(null);
                    setPanel(null);
                  })
                }
              />
            )}
            {panel === "profiles" && (
              <ProfilesPanel snapshot={snapshot} onClose={closePanel} onRefresh={refresh} onSwitch={handleSwitchProfile} />
            )}
          </div>
        </div>
      </div>

      {toast && (
        <div className="dl-toast" role="status">
          <Check size={16} color="var(--ok)" />
          <div>
            <div className="dl-toast-title">İndirme tamamlandı</div>
            <div className="dl-toast-sub">{toast}</div>
          </div>
        </div>
      )}
    </div>
  );
}
