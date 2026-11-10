// QuantumBrowser — Rust çekirdeğiyle konuşan tipli IPC köprüsü
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Bookmark,
  DownloadEntry,
  HistoryEntry,
  Profile,
  Snapshot,
} from "./types";

// ---------- Anlık görüntü & sekmeler ----------
export const getSnapshot = () => invoke<Snapshot>("get_snapshot");
export const newTab = (url?: string, incognito = false) =>
  invoke<void>("new_tab", { url: url ?? null, incognito });
export const newIncognitoTab = () => invoke<void>("new_incognito_tab");
export const closeTab = (id: string) => invoke<void>("close_tab", { id });
export const activateTab = (id: string) => invoke<void>("activate_tab", { id });
export const duplicateTab = (id: string) => invoke<void>("duplicate_tab", { id });
export const moveTab = (fromId: string, toId: string) =>
  invoke<void>("move_tab", { fromId, toId });
export const togglePin = (id: string) => invoke<void>("toggle_pin", { id });
export const closeOthers = (id: string) => invoke<void>("close_others", { id });
export const navigate = (url: string) => invoke<void>("navigate", { url });
export const goBack = () => invoke<void>("go_back");
export const goForward = () => invoke<void>("go_forward");
export const reload = () => invoke<void>("reload");

// ---------- Gruplar ----------
export const createGroup = (name: string, color: string) =>
  invoke<number>("create_group", { name, color });
export const toggleGroup = (groupId: number) => invoke<void>("toggle_group", { groupId });
export const deleteGroup = (groupId: number) => invoke<void>("delete_group", { groupId });
export const assignGroup = (tabId: string, groupId: number | null) =>
  invoke<void>("assign_group", { tabId, groupId });

// ---------- Kabuk ----------
export const setOverlayOpen = (open: boolean) => invoke<void>("set_overlay_open", { open });
export const setSidebarCollapsed = (collapsed: boolean) =>
  invoke<void>("set_sidebar_collapsed", { collapsed });

// ---------- Ayarlar ----------
export const setTheme = (theme: string) => invoke<void>("set_theme", { theme });
export const setSearchEngine = (engine: string) => invoke<void>("set_search_engine", { engine });
export const setBlockerEnabled = (enabled: boolean) =>
  invoke<void>("set_blocker_enabled", { enabled });

// ---------- Yer imleri ----------
export const listBookmarks = (query?: string) =>
  invoke<Bookmark[]>("list_bookmarks", { query: query ?? null });
export const toggleBookmark = (url: string, title: string) =>
  invoke<boolean>("toggle_bookmark", { url, title });
export const isBookmarked = (url: string) => invoke<boolean>("is_bookmarked", { url });
export const removeBookmark = (url: string) => invoke<void>("remove_bookmark", { url });

// ---------- Geçmiş ----------
export const listHistory = (query?: string, limit = 200) =>
  invoke<HistoryEntry[]>("list_history", { query: query ?? null, limit });
export const clearHistory = () => invoke<void>("clear_history");

// ---------- İndirmeler ----------
export const listDownloads = () => invoke<DownloadEntry[]>("list_downloads");
export const clearFinishedDownloads = () => invoke<void>("clear_finished_downloads");
export const openDownloadsFolder = () => invoke<void>("open_downloads_folder");

// ---------- Profiller ----------
export const listProfiles = () => invoke<Profile[]>("list_profiles");
export const addProfile = (name: string, color: string) =>
  invoke<number>("add_profile", { name, color });
export const switchProfile = (profileId: number) =>
  invoke<void>("switch_profile", { profileId });

// ---------- Gizlilik ----------
export const clearBrowsingData = () => invoke<void>("clear_browsing_data");

// ---------- Olaylar ----------
export const onTabsChanged = (fn: () => void): Promise<UnlistenFn> =>
  listen("tabs-changed", fn);
export const onBlockerCount = (
  fn: (payload: { id: string; count: number }) => void
): Promise<UnlistenFn> =>
  listen<{ id: string; count: number }>("blocker-count", (e) => fn(e.payload));
export const onDownloadsChanged = (fn: () => void): Promise<UnlistenFn> =>
  listen("downloads-changed", fn);

// ---------- Biçimleyiciler ----------
export function formatBytes(n: number): string {
  if (n < 0) return "—";
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB"];
  let value = n / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${units[unit]}`;
}

export function domainOf(url: string): string {
  try {
    const u = new URL(url);
    return u.hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

export function historyGroup(visitedAt: string): string {
  // visited_at: "YYYY-MM-DD HH:MM:SS" (yerel saat)
  const day = visitedAt.slice(0, 10);
  const today = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  const todayStr = `${today.getFullYear()}-${pad(today.getMonth() + 1)}-${pad(today.getDate())}`;
  if (day === todayStr) return "Bugün";
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);
  const yStr = `${yesterday.getFullYear()}-${pad(yesterday.getMonth() + 1)}-${pad(yesterday.getDate())}`;
  if (day === yStr) return "Dün";
  const [y, m, d] = day.split("-").map(Number);
  return new Date(y, m - 1, d).toLocaleDateString("tr-TR", {
    day: "numeric",
    month: "long",
    year: "numeric",
  });
}

export function timeOf(visitedAt: string): string {
  return visitedAt.slice(11, 16);
}
