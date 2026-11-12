// QuantumBrowser — paylaşılan tipler (Rust tarafıyla birebir)

export interface TabMeta {
  id: string; // webview etiketi ("tab-<uuid>")
  url: string;
  title: string;
  group_id: number | null;
  pinned: boolean;
  incognito: boolean;
  blocked_count: number;
  is_newtab: boolean;
  kind: string | null; // null = harici site; "newtab" | "settings" | ...
}

export interface TabGroup {
  id: number;
  profile_id: number;
  name: string;
  color: string;
  collapsed: boolean;
  position: number;
}

export interface Profile {
  id: number;
  name: string;
  color: string;
  store_id: string;
  is_default: boolean;
}

export interface Snapshot {
  tabs: TabMeta[];
  active_id: string | null;
  groups: TabGroup[];
  profile_id: number | null;
  profile_name: string;
  profile_color: string;
  profiles: Profile[];
  theme: string;
  search_engine: string;
  blocker_enabled: boolean;
  sidebar_collapsed: boolean;
  session_restore: boolean;
  accent: string;
  block_insecure: boolean;
  auto_update: boolean;
}

export interface Bookmark {
  id: number;
  profile_id: number;
  url: string;
  title: string;
  created_at: string;
}

export interface HistoryEntry {
  id: number;
  profile_id: number;
  url: string;
  title: string;
  visited_at: string;
}

export interface DownloadEntry {
  id: number;
  profile_id: number;
  url: string;
  file_name: string;
  file_path: string;
  bytes_received: number;
  bytes_total: number;
  state: string; // downloading | completed | interrupted
  started_at: string;
  finished_at: string | null;
}
