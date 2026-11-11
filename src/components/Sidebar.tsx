import { useState, useEffect } from "react";
import {
  Atom,
  Bookmark,
  ChevronDown,
  ChevronRight,
  Copy,
  Download,
  EyeOff,
  FolderMinus,
  FolderPlus,
  History,
  Pin,
  PinOff,
  Plus,
  Settings,
  ShieldCheck,
  X,
  PanelLeftClose,
  PanelLeftOpen,
  MoreVertical,
  Users,
} from "lucide-react";

const KIND_GLYPHS: Record<string, typeof Settings> = {
  newtab: Atom,
  settings: Settings,
  history: History,
  bookmarks: Bookmark,
  downloads: Download,
  profiles: Users,
};
import type { TabMeta, Snapshot } from "../types";
import ContextMenu, { type MenuEntry } from "./ContextMenu";

export type Panel = "history" | "bookmarks" | "downloads" | "settings" | "profiles";

interface Props {
  snapshot: Snapshot;
  onToggleSidebar: () => void;
  onNewTab: () => void;
  onNewIncognito: () => void;
  onClose: (id: string) => void;
  onActivate: (id: string) => void;
  onDuplicate: (id: string) => void;
  onTogglePin: (id: string) => void;
  onCloseOthers: (id: string) => void;
  onMoveTab: (fromId: string, toId: string) => void;
  onToggleGroup: (groupId: number) => void;
  onAssignGroup: (tabId: string, groupId: number | null) => void;
  onCreateGroupAndAssign: (tabId: string, name: string, color: string) => void;
  onOpenPanel: (panel: Panel) => void;
  onMenuChange?: (open: boolean) => void;
}

function hueFromString(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) >>> 0;
  return h % 360;
}

export function LetterAvatar({
  url,
  isNewTab,
  incognito,
  kind,
}: {
  url: string;
  isNewTab: boolean;
  incognito: boolean;
  kind?: string | null;
}) {
  if (kind && KIND_GLYPHS[kind]) {
    const Icon = KIND_GLYPHS[kind];
    return (
      <span className="letter-avatar" style={{ background: "var(--accent-grad)" }}>
        <Icon size={12} strokeWidth={2} />
      </span>
    );
  }
  if (isNewTab)
    return (
      <span className="letter-avatar" style={{ background: "var(--accent-grad)" }}>
        <Atom size={13} strokeWidth={2} />
      </span>
    );
  if (incognito)
    return (
      <span className="letter-avatar" style={{ background: "#6d28d9" }}>
        <EyeOff size={12} strokeWidth={2} />
      </span>
    );
  try {
    const host = new URL(url).hostname.replace(/^www\./, "");
    const letter = host.charAt(0).toUpperCase() || "?";
    const hue = hueFromString(host);
    return (
      <span className="letter-avatar" style={{ background: `hsl(${hue} 55% 46%)` }}>
        {letter}
      </span>
    );
  } catch {
    return <span className="letter-avatar favicon-empty">?</span>;
  }
}

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  return (parts[0]?.[0] ?? "?").toUpperCase() + (parts[1]?.[0] ?? "").toUpperCase();
}

export default function Sidebar(props: Props) {
  const { snapshot } = props;
  const collapsed = snapshot.sidebar_collapsed;
  const [menu, setMenu] = useState<{ x: number; y: number; tabId: string } | null>(null);
  const [dragId, setDragId] = useState<string | null>(null);
  const [overId, setOverId] = useState<string | null>(null);

  // Menü açık/kapalı bilgisini üst yönetime bildir (çocuk webview'ı küçültmek için)
  const { onMenuChange } = props;
  useEffect(() => {
    onMenuChange?.(menu !== null);
  }, [menu, onMenuChange]);

  const pinned = snapshot.tabs.filter((t) => t.pinned);
  const rest = snapshot.tabs.filter((t) => !t.pinned);
  const grouped = new Map<number, TabMeta[]>();
  for (const t of rest) {
    if (t.group_id != null) {
      const list = grouped.get(t.group_id) ?? [];
      list.push(t);
      grouped.set(t.group_id, list);
    }
  }
  const ungrouped = rest.filter((t) => t.group_id == null);

  const menuEntries = (tab: TabMeta): MenuEntry[] => {
    const entries: MenuEntry[] = [
      { label: "Sekmeyi Çoğalt", icon: Copy, onSelect: () => props.onDuplicate(tab.id) },
      {
        label: tab.pinned ? "Sabitlemeyi Kaldır" : "Sekmeyi Sabitle",
        icon: tab.pinned ? PinOff : Pin,
        onSelect: () => props.onTogglePin(tab.id),
      },
      "separator",
    ];
    const groupsWithTabs = snapshot.groups;
    if (groupsWithTabs.length > 0) {
      for (const g of groupsWithTabs) {
        const inThis = tab.group_id === g.id;
        entries.push({
          label: inThis ? `"${g.name}" Grubundan Çıkar` : `"${g.name}" Grubuna Ekle`,
          icon: inThis ? FolderMinus : FolderPlus,
          onSelect: () => props.onAssignGroup(tab.id, inThis ? null : g.id),
        });
      }
    } else {
      entries.push({
        label: "Yeni Grup Oluştur ve Ekle (Genel)",
        icon: FolderPlus,
        onSelect: () => props.onCreateGroupAndAssign(tab.id, "Genel", "#0ea5e9"),
      });
    }
    entries.push(
      "separator",
      { label: "Diğerlerini Kapat", icon: X, onSelect: () => props.onCloseOthers(tab.id) },
      { label: "Sekmeyi Kapat", icon: X, danger: true, onSelect: () => props.onClose(tab.id) }
    );
    return entries;
  };

  const renderTab = (tab: TabMeta) => {
    const active = snapshot.active_id === tab.id;
    const over = overId === tab.id && dragId !== tab.id;
    const displayTitle = tab.is_newtab ? "Yeni Sekme" : tab.title || tab.url;
    return (
      <button
        key={tab.id}
        className={`tab${active ? " active" : ""}${tab.pinned ? " pinned" : ""}${over ? " drag-over" : ""}`}
        title={displayTitle}
        onClick={() => props.onActivate(tab.id)}
        onAuxClick={(e) => {
          if (e.button === 1) {
            e.preventDefault();
            props.onClose(tab.id);
          }
        }}
        onContextMenu={(e) => {
          e.preventDefault();
          setMenu({ x: e.clientX, y: e.clientY, tabId: tab.id });
        }}
        draggable
        onDragStart={(e) => {
          setDragId(tab.id);
          e.dataTransfer.effectAllowed = "move";
        }}
        onDragOver={(e) => {
          e.preventDefault();
          if (overId !== tab.id) setOverId(tab.id);
        }}
        onDrop={(e) => {
          e.preventDefault();
          if (dragId && dragId !== tab.id) props.onMoveTab(dragId, tab.id);
          setDragId(null);
          setOverId(null);
        }}
        onDragEnd={() => {
          setDragId(null);
          setOverId(null);
        }}
      >
        {tab.pinned && <span className="pin-strip" aria-hidden="true" />}
        <LetterAvatar url={tab.url} isNewTab={tab.is_newtab} incognito={tab.incognito} kind={tab.kind} />
        <span className="tab-title">{displayTitle}</span>
        {tab.blocked_count > 0 && (
          <span className="badge" title={`${tab.blocked_count} reklam/izleyici engellendi`}>
            <ShieldCheck size={10} strokeWidth={2.2} /> {tab.blocked_count}
          </span>
        )}
        <span
          className="tab-more"
          role="button"
          aria-label="Sekme seçenekleri"
          title="Seçenekler"
          onClick={(e) => {
            e.stopPropagation();
            const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
            setMenu({ x: rect.left, y: rect.bottom + 4, tabId: tab.id });
          }}
        >
          <MoreVertical size={13} strokeWidth={2} />
        </span>
        <span
          className="tab-close"
          role="button"
          aria-label="Sekmeyi kapat"
          title="Kapat"
          onClick={(e) => {
            e.stopPropagation();
            props.onClose(tab.id);
          }}
        >
          <X size={13} strokeWidth={2} />
        </span>
      </button>
    );
  };

  const activeMenuTab = menu ? snapshot.tabs.find((t) => t.id === menu.tabId) : null;

  return (
    <aside className={`sidebar${collapsed ? " collapsed" : ""}`} aria-label="Sekmeler">
      <div className="sidebar-head">
        <span className="side-label">SEKMELER</span>
        <button
          className="icon-btn"
          onClick={props.onToggleSidebar}
          title={collapsed ? "Şeridi Genişlet" : "Şeridi Daralt"}
          aria-label={collapsed ? "Şeridi genişlet" : "Şeridi daralt"}
        >
          {collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
        </button>
      </div>

      <div className="tabs-scroll">
        {pinned.length > 0 && (
          <div className="section-title">
            <Pin size={10} strokeWidth={2.5} /> Sabitlenenler
          </div>
        )}
        {pinned.map(renderTab)}

        {snapshot.groups.map((group) => {
          const tabs = grouped.get(group.id) ?? [];
          return (
            <div key={group.id}>
              <button className="group-chip" onClick={() => props.onToggleGroup(group.id)} title={group.collapsed ? "Grubu aç" : "Grubu daralt"}>
                <span className="group-dot" style={{ background: group.color }} />
                <span className="group-name">{group.name}</span>
                <span className="group-count">{tabs.length}</span>
                <ChevronRight size={12} strokeWidth={2.4} className={`group-chevron${group.collapsed ? "" : " open"}`} />
              </button>
              {!group.collapsed && <div className="tabs-nested">{tabs.map(renderTab)}</div>}
            </div>
          );
        })}

        {ungrouped.length > 0 && snapshot.groups.length > 0 && pinned.length === 0 && (
          <div className="section-title">Diğerleri</div>
        )}
        {ungrouped.map(renderTab)}
      </div>

      <div className="sidebar-foot">
        <button className="btn-newtab" onClick={props.onNewTab} title="Yeni Sekme (Ctrl+T)">
          <Plus size={16} strokeWidth={2.6} />
          <span>Yeni Sekme</span>
        </button>

        <div className="side-actions">
          <button className="icon-btn" onClick={props.onNewIncognito} title="Gizli Sekme (Ctrl+Shift+N)" aria-label="Gizli sekme">
            <EyeOff size={16} strokeWidth={2} />
          </button>
          <button className="icon-btn" onClick={() => props.onOpenPanel("downloads")} title="İndirmeler (Ctrl+J)" aria-label="İndirmeler">
            <Download size={16} strokeWidth={2} />
          </button>
          <button className="icon-btn" onClick={() => props.onOpenPanel("bookmarks")} title="Yer İmleri (Ctrl+B)" aria-label="Yer imleri">
            <Bookmark size={16} strokeWidth={2} />
          </button>
          <button className="icon-btn" onClick={() => props.onOpenPanel("history")} title="Geçmiş (Ctrl+Shift+Y)" aria-label="Geçmiş">
            <History size={16} strokeWidth={2} />
          </button>
          <span style={{ flex: 1 }} />
          <button className="icon-btn" onClick={() => props.onOpenPanel("settings")} title="Ayarlar" aria-label="Ayarlar">
            <Settings size={16} strokeWidth={2} />
          </button>
        </div>

        <button className="profile-foot" onClick={() => props.onOpenPanel("profiles")} title="Profili değiştir">
          <span className="avatar" style={{ background: snapshot.profile_color }}>
            {initials(snapshot.profile_name)}
          </span>
          <span className="profile-foot-meta">
            <span className="profile-foot-name">{snapshot.profile_name}</span>
            <span className="profile-foot-hint">Profili değiştir</span>
          </span>
          <ChevronDown size={14} strokeWidth={2} />
        </button>
      </div>

      {menu && activeMenuTab && (
        <ContextMenu x={menu.x} y={menu.y} entries={menuEntries(activeMenuTab)} onClose={() => setMenu(null)} />
      )}
    </aside>
  );
}
