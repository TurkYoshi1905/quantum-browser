import { useEffect, useMemo, useState } from "react";
import {
  Search,
  Plus,
  EyeOff,
  RotateCw,
  X,
  Copy,
  Bookmark,
  History,
  Download,
  Users,
  Settings,
  Moon,
  Sun,
  Monitor,
  Trash2,
  ShieldAlert,
  PanelLeftClose,
  KeyRound,
  ExternalLink,
  ArrowRight,
  Palette as PaletteIcon,
} from "lucide-react";
import * as api from "../api";
import type { Snapshot } from "../types";
import { applyRootTheme } from "../utils/theme";

interface Cmd {
  id: string;
  title: string;
  hint?: string;
  keywords?: string;
  icon: typeof Plus;
  run: () => void | Promise<void>;
}

const norm = (s: string) => s.toLowerCase();
const matches = (c: Cmd, q: string) =>
  !q || norm(c.title).includes(q) || (c.keywords ? norm(c.keywords).includes(q) : false);

export default function Palette() {
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const [snap, setSnap] = useState<Snapshot | null>(null);

  useEffect(() => {
    api.getSnapshot()
      .then((s) => {
        setSnap(s);
        applyRootTheme(s.theme || "light", s.accent || "mor");
      })
      .catch(() => {});
  }, []);

  const cmds = useMemo<Cmd[]>(() => {
    const close = () => api.hidePalette().catch(() => {});
    const closeAnd = (fn: () => void | Promise<void>) => () => {
      fn();
      close();
    };
    const openInternal = (kind: string) => closeAnd(() => api.newTab(`quantum://${kind}`));
    return [
      { id: "newtab", title: "Yeni Sekme", keywords: "yeni sekme tab new", icon: Plus, run: () => api.newTab().then(close) },
      { id: "incognito", title: "Gizli Sekme", keywords: "gizli incognito yeni", icon: EyeOff, run: () => api.newIncognitoTab().then(close) },
      { id: "reload", title: "Sayfayı Yenile", keywords: "yenile reload", icon: RotateCw, run: closeAnd(api.reload) },
      { id: "close-tab", title: "Aktif Sekmeyi Kapat", keywords: "kapat close sekme", icon: X, run: () => { void api.getSnapshot().then((s) => { if (s.active_id) api.closeTab(s.active_id); }); close(); } },
      { id: "duplicate", title: "Sekmeyi Çoğalt", keywords: "çoğalt duplicate", icon: Copy, run: () => { void api.getSnapshot().then((s) => { if (s.active_id) api.duplicateTab(s.active_id); }); close(); } },
      { id: "reopen", title: "Kapatılan Sekmeyi Geri Aç", keywords: "geri aç ctrl+shift+t reopen", icon: ArrowRight, run: closeAnd(api.reopenClosedTab) },
      { id: "new-history", title: "Geçmiş", keywords: "geçmiş history", icon: History, run: openInternal("history") },
      { id: "new-bookmarks", title: "Yer İmleri", keywords: "yer imleri bookmarks", icon: Bookmark, run: openInternal("bookmarks") },
      { id: "new-downloads", title: "İndirilenler", keywords: "indirilenler downloads", icon: Download, run: openInternal("downloads") },
      { id: "new-profiles", title: "Profiller", keywords: "profiller profiles kullanıcı", icon: Users, run: openInternal("profiles") },
      { id: "new-vault", title: "Quantum Şifre Kasası", keywords: "şifre kasa parola vault password", icon: KeyRound, run: openInternal("vault") },
      { id: "new-settings", title: "Ayarlar", keywords: "ayarlar settings", icon: Settings, run: openInternal("settings") },
      { id: "theme-dark", title: "Koyu Tema", keywords: "koyu dark tema", icon: Moon, run: closeAnd(() => api.setTheme("dark")) },
      { id: "theme-light", title: "Açık Tema", keywords: "açık light tema", icon: Sun, run: closeAnd(() => api.setTheme("light")) },
      { id: "theme-system", title: "Sistem Teması", keywords: "sistem system tema", icon: Monitor, run: closeAnd(() => api.setTheme("system")) },
      { id: "toggle-sidebar", title: "Yan Çubuğu Daralt / Genişlet", keywords: "yan çubuk sidebar daralt", icon: PanelLeftClose, run: () => { void api.getSnapshot().then((s) => api.setSidebarCollapsed(!s.sidebar_collapsed)); close(); } },
      { id: "clear-data", title: "Geçmişi ve İndirme Listesini Temizle", keywords: "temizle geçmiş sil clear", icon: Trash2, run: closeAnd(() => api.clearBrowsingData()) },
      { id: "blocker", title: "Reklam Engelleyiciyi Aç/Kapat", keywords: "engelle reklam engelleyici blocker", icon: ShieldAlert, run: () => { void api.getSnapshot().then((s) => api.setBlockerEnabled(!s.blocker_enabled)); close(); } },
      { id: "update", title: "Güncellemeleri Denetle", keywords: "güncelle update check", icon: ExternalLink, run: closeAnd(() => api.newTab("quantum://settings")) },
    ];
  }, []);

  const filtered = useMemo(
    () => cmds.filter((c) => matches(c, norm(query.trim()))),
    [cmds, query]
  );
  useEffect(() => { setSelected(0); }, [query]);

  const runSelected = () => {
    const cmd = filtered[selected];
    if (cmd) cmd.run();
    else api.newTab().catch(() => {});
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") { e.preventDefault(); api.hidePalette().catch(() => {}); }
    else if (e.key === "ArrowDown") { e.preventDefault(); setSelected((v) => Math.min(v + 1, filtered.length - 1)); }
    else if (e.key === "ArrowUp") { e.preventDefault(); setSelected((v) => Math.max(v - 1, 0)); }
    else if (e.key === "Enter") { e.preventDefault(); runSelected(); }
  };

  return (
    <main className="palette-backdrop" onClick={() => api.hidePalette().catch(() => {})} onKeyDown={onKeyDown}>
      <div className="palette-panel" onClick={(e) => e.stopPropagation()}>
        <div className="palette-input-row">
          <Search size={17} strokeWidth={2} aria-hidden="true" />
          <input
            autoFocus
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Komut ara… (örn. yeni sekme, koyu tema, güncelle)"
            aria-label="Komut paleti"
            spellCheck={false}
          />
          <span className="palette-hint">Esc</span>
        </div>
        <div className="palette-list" role="listbox" aria-label="Komutlar">
          {filtered.length === 0 && <div className="palette-empty">Sonuç bulunamadı</div>}
          {filtered.map((cmd, i) => {
            const Icon = cmd.icon;
            const on = i === selected;
            return (
              <button
                key={cmd.id}
                className={`palette-item${on ? " on" : ""}`}
                role="option"
                aria-selected={on}
                onMouseEnter={() => setSelected(i)}
                onClick={() => cmd.run()}
              >
                <span className="palette-icon"><Icon size={15} strokeWidth={2} /></span>
                <span className="palette-title">{cmd.title}</span>
                <span className="palette-kbd">↵</span>
              </button>
            );
          })}
        </div>
        <div className="palette-foot">
          <span><span className="palette-kbd">↑↓</span> gezin</span>
          <span><span className="palette-kbd">↵</span> çalıştır</span>
          <span><span className="palette-kbd">Esc</span> kapat</span>
        </div>
      </div>
    </main>
  );
}
