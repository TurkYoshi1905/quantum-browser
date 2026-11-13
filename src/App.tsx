import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "./api";
import type { Snapshot } from "./types";
import { applyRootTheme } from "./utils/theme";
import TitleBar from "./components/TitleBar";
import Sidebar from "./components/Sidebar";
import AddressBar from "./components/AddressBar";

const INTERNAL_ICONS_HINT = ["newtab", "settings", "history", "bookmarks", "downloads", "profiles"];

export default function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
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

  // İndirme tamamlanınca baloncuk (çocuk webview — sayfa yok edilmez)
  const seenDownloads = useRef<Set<number>>(new Set());
  useEffect(() => {
    const un = api.onDownloadsChanged(async () => {
      try {
        const items = await api.listDownloads();
        for (const d of items) {
          if (d.state === "completed" && !seenDownloads.current.has(d.id)) {
            seenDownloads.current.add(d.id);
            api.showToast(d.file_name).catch(() => {});
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
  const accent = snapshot?.accent;
  useEffect(() => {
    if (!theme) return;
    applyRootTheme(theme, accent || "mor");
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => applyRootTheme(theme, accent || "mor");
    mq.addEventListener?.("change", onChange);
    return () => mq.removeEventListener?.("change", onChange);
  }, [theme, accent]);

  // Popover odağı çalmadığı için menü açıkken kabuk prompt dinleyicisi burada yaşar:
  // Escape veya menü dışına mousedown → popover kapanır (kabuğun hiçbir butonu kilitlenmez).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") api.closeMenuPopup().catch(() => {});
    };
    const onMouseDown = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      if (target?.closest("[data-menu-anchor]")) return; // ⋮ butonu toggle'ı kendi halletsin
      api.closeMenuPopup().catch(() => {});
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("mousedown", onMouseDown);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("mousedown", onMouseDown);
    };
  }, []);

  const activeTab = snapshot?.tabs.find((t) => t.id === snapshot.active_id) ?? null;
  const activeUrl = activeTab?.url ?? "";
  const isActiveNewTab = activeTab?.is_newtab ?? true;
  const isActiveIncognito = activeTab?.incognito ?? false;
  const internalKind = activeTab?.kind ?? null;

  const openInternal = useCallback((kind: string) => {
    api.newTab(`quantum://${kind}`);
  }, []);

  // Kitap yıldızı durumu (dahili sayfalarda gizli)
  const [bookmarked, setBookmarked] = useState(false);
  useEffect(() => {
    if (!activeUrl || isActiveNewTab || internalKind) {
      setBookmarked(false);
      return;
    }
    api.isBookmarked(activeUrl).then(setBookmarked).catch(() => setBookmarked(false));
  }, [activeUrl, isActiveNewTab, internalKind]);

  // Klavye kısayolları (kabuk odaktayken)
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // F12 — geliştirici panelini aç/kapat
      if (e.key === "F12" || ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "i")) {
        e.preventDefault();
        api.toggleDevtools();
        return;
      }

      const mod = e.ctrlKey || e.metaKey;
      const alt = e.altKey;
      const shift = e.shiftKey;
      const key = e.key.toLowerCase();

      if (key === "escape") return;
      if (key === "f5") {
        e.preventDefault();
        api.reload();
        return;
      }
      if (alt && key === "arrowleft") return api.goBack();
      if (alt && key === "arrowright") return api.goForward();
      if (!mod) return;

      const run = (fn: () => void) => {
        e.preventDefault();
        fn();
      };

      if (key === "k" && !shift) {
        e.preventDefault();
        api.togglePalette();
        return;
      }
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

      if (key === "t" && !shift) return run(() => api.newTab());
      if (key === "t" && shift) return run(() => api.reopenClosedTab());
      if (key === "n" && shift) return run(() => api.newIncognitoTab());
      if (key === "w") {
        const id = snapshotRef.current?.active_id;
        if (id) return run(() => api.closeTab(id));
        return;
      }
      if (key === "l" || key === "k") return run(() => omniboxRef.current?.focus());
      if (key === "r") return run(() => api.reload());
      if (key === "b" && !shift) return run(() => openInternal("bookmarks"));
      if (key === "j") return run(() => openInternal("downloads"));
      if (key === ",") return run(() => openInternal("settings"));
      if (key === "y" && shift) return run(() => openInternal("history"));
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [openInternal]);

  const cycleTab = useCallback((direction: 1 | -1) => {
    const snap = snapshotRef.current;
    if (!snap || snap.tabs.length === 0) return;
    const idx = snap.tabs.findIndex((t) => t.id === snap.active_id);
    const next = (idx + direction + snap.tabs.length) % snap.tabs.length;
    api.activateTab(snap.tabs[next].id);
  }, []);

  const activateNth = useCallback((n: number) => {
    const snap = snapshotRef.current;
    if (!snap) return;
    const target = n === 9 ? snap.tabs[snap.tabs.length - 1] : snap.tabs[n - 1];
    if (target) api.activateTab(target.id);
  }, []);

  const handleNavigate = useCallback((input: string) => {
    const trimmed = input.trim();
    if (trimmed) api.navigate(trimmed);
  }, []);

  const handleToggleBookmark = useCallback(async () => {
    const snap = snapshotRef.current;
    const tab = snap?.tabs.find((t) => t.id === snap.active_id);
    if (!tab || tab.is_newtab || !tab.url || tab.kind) return;
    try {
      const now = await api.toggleBookmark(tab.url, tab.title || tab.url);
      // yer iminden başka UI yok; snapshot yenileyelim
      api.getSnapshot().then(setSnapshot);
    } catch {
      /* sessiz */
    }
  }, []);

  const handleToggleBlocker = useCallback(() => {
    const snap = snapshotRef.current;
    if (!snap) return;
    api.setBlockerEnabled(!snap.blocker_enabled);
  }, []);

  if (!snapshot) {
    return (
      <div className="shell" style={{ gridTemplateRows: "1fr" }}>
        <div className="newtab" style={{ minHeight: "100vh" }}>
          <img className="splash-img" src="/quantum-browser.png" alt="QuantumBrowser" width={96} height={96} />
          <div className="greeting" style={{ marginTop: 10 }}>QuantumBrowser yükleniyor…</div>
        </div>
      </div>
    );
  }

  // Sessiz güncelleme kontrolü (ayarlıysa) — dişlide mor rozet + toast + arka planda indirme
  const [updateDot, setUpdateDot] = useState(false);
  useEffect(() => {
    if (!snapshot?.auto_update) return;
    const background = snapshot.auto_update_background;
    let active = true;
    import("@tauri-apps/plugin-updater")
      .then(({ check }) => check())
      .then((update) => {
        if (!active || !update) return;
        setUpdateDot(true);
        if (background) {
          update
            .download()
            .then(() => { if (active) api.showToast(`Güncelleme indirildi: v${update.version} — Ayarlar'dan kur`).catch(() => {}); })
            .catch(() => {});
        } else {
          api.showToast(`Güncelleme hazır: v${update.version} — Ayarlar → Güncelleme`).catch(() => {});
        }
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [snapshot?.auto_update, snapshot?.auto_update_background]);

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
          onCreateGroupAndAssign={(tabId, name, color) =>
            api.createGroup(name, color).then((groupId) => api.assignGroup(tabId, groupId)).then(refresh)
          }
          onOpenPanel={openInternal}
          updateDot={updateDot}
        />

        <div className="content">
          <AddressBar
            url={activeUrl}
            isIncognito={isActiveIncognito}
            isNewTab={isActiveNewTab}
            internalKind={internalKind}
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
            onOpenPanel={openInternal}
          />

          <div className="page-area">
            <div className="page-empty" aria-hidden="true">
              <span style={{ opacity: 0.35 }}>QuantumBrowser</span>
            </div>
          </div>
        </div>
      </div>

    </div>
  );
}
