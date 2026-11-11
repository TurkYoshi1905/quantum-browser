import { useCallback, useEffect, useState, useRef } from "react";
import {
  AlertTriangle,
  Bookmark,
  Check,
  CheckCircle2,
  Download,
  FileClock,
  FolderOpen,
  Globe,
  History,
  Info,
  Moon,
  Monitor,
  Pencil,
  Plus,
  Search,
  Settings2,
  ShieldCheck,
  Sun,
  Trash2,
  Users,
  X,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import type { Panel } from "./Sidebar";
import type { Snapshot } from "../types";
import {
  addProfile,
  clearBrowsingData,
  clearFinishedDownloads,
  clearHistory,
  deleteHistoryEntries,
  domainOf,
  formatBytes,
  historyGroup,
  listBookmarks,
  listDownloads,
  listHistory,
  listProfiles,
  navigate,
  onDownloadsChanged,
  openDownloadsFolder,
  removeBookmark,
  renameProfile,
  deleteProfile,
  timeOf,
} from "../api";
import type { Bookmark as BookmarkType, DownloadEntry, HistoryEntry } from "../types";

// ---------- Kabuk ----------
function PanelShell({
  title,
  icon: Icon,
  onClose,
  children,
  actions,
}: {
  title: string;
  icon: LucideIcon;
  onClose: () => void;
  children: React.ReactNode;
  actions?: React.ReactNode;
}) {
  return (
    <div className="panel" role="dialog" aria-label={title}>
      <div className="panel-head">
        <h1>
          <Icon size={19} strokeWidth={1.8} /> {title}
        </h1>
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          {actions}
          <button className="panel-close" onClick={onClose} aria-label="Kapat (Esc)" title="Kapat (Esc)">
            <X size={18} />
          </button>
        </div>
      </div>
      {children}
    </div>
  );
}

// ---------- İki adımlı onay (pencere.confirm WebView2'de yok) ----------
function ArmedButton({
  label,
  confirmLabel,
  onConfirm,
  danger = true,
}: {
  label: string;
  confirmLabel: string;
  onConfirm: () => void;
  danger?: boolean;
}) {
  const [armed, setArmed] = useState(false);
  useEffect(() => {
    if (!armed) return;
    const t = window.setTimeout(() => setArmed(false), 4000);
    return () => window.clearTimeout(t);
  }, [armed]);
  if (!armed)
    return (
      <button className={`btn small${danger ? " danger" : ""}`} onClick={() => setArmed(true)}>
        {label}
      </button>
    );
  return (
    <button
      className="btn small danger"
      autoFocus
      onClick={() => {
        setArmed(false);
        onConfirm();
      }}
    >
      {confirmLabel}
    </button>
  );
}

// ---------- Geçmiş ----------
export function HistoryPanel({ onClose, onNavigate }: { onClose: () => void; onNavigate: (url: string) => void }) {
  const [query, setQuery] = useState("");
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [loaded, setLoaded] = useState(false);
  const selectAllRef = useRef<HTMLInputElement>(null);

  const load = useCallback((q?: string) => {
    listHistory(q?.trim() || undefined, 300).then((rows) => {
      setEntries(rows);
      setLoaded(true);
    });
  }, []);

  useEffect(() => {
    const t = window.setTimeout(() => {
      load(query);
      setSelected(new Set());
    }, 150);
    return () => window.clearTimeout(t);
  }, [query, load]);

  const allVisibleIds = entries.map((e) => e.id);
  const allChecked =
    entries.length > 0 && allVisibleIds.every((id) => selected.has(id));
  const someChecked = selected.size > 0 && !allChecked;

  useEffect(() => {
    if (selectAllRef.current) selectAllRef.current.indeterminate = someChecked;
  }, [someChecked]);

  const toggleAll = (checked: boolean) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (checked) allVisibleIds.forEach((id) => next.add(id));
      else allVisibleIds.forEach((id) => next.delete(id));
      return next;
    });
  };

  const toggleOne = (id: number) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const deleteSelected = () => {
    const ids = [...selected];
    if (ids.length === 0) return;
    deleteHistoryEntries(ids).then(() => {
      setSelected(new Set());
      load(query);
    });
  };

  const grouped = new Map<string, HistoryEntry[]>();
  for (const e of entries) {
    const key = historyGroup(e.visited_at);
    const list = grouped.get(key) ?? [];
    list.push(e);
    grouped.set(key, list);
  }

  return (
    <PanelShell
      title="Geçmiş"
      icon={History}
      onClose={onClose}
      actions={
        <ArmedButton
          label={`Seçilenleri Sil (${selected.size})`}
          confirmLabel="Kesinlikle Sil"
          onConfirm={deleteSelected}
        />
      }
    >
      <div className="panel-search">
        <Search size={15} />
        <input
          autoFocus
          placeholder="Geçmişte ara…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          aria-label="Geçmişte ara"
        />
      </div>

      {loaded && entries.length > 0 && (
        <label className="select-all-row" aria-label="Tümünü seç">
          <input
            ref={selectAllRef}
            type="checkbox"
            className="row-check"
            checked={allChecked}
            onChange={(e) => toggleAll(e.target.checked)}
            aria-label="Tümünü seç"
          />
          <span>{allChecked ? "Seçimi kaldır" : "Tümünü seç"} ({entries.length})</span>
          {selected.size > 0 && <span className="selected-count">{selected.size} seçili</span>}
        </label>
      )}

      {loaded && entries.length === 0 && (
        <div className="empty-state">
          <History size={38} strokeWidth={1.5} />
          <p>{query ? "Sonuç bulunamadı." : "Henüz geçmiş yok — birkaç sayfa gezinin."}</p>
        </div>
      )}
      {[...grouped.entries()].map(([group, rows]) => (
        <div key={group}>
          <div className="group-label">{group}</div>
          {rows.map((e) => (
            <div key={e.id} className="row-item history-row">
              <input
                type="checkbox"
                className="row-check"
                checked={selected.has(e.id)}
                onChange={() => toggleOne(e.id)}
                aria-label={`Seç: ${e.title || domainOf(e.url)}`}
                onClick={(ev) => ev.stopPropagation()}
              />
              <button className="row-item" style={{ flex: 1, padding: 0, alignItems: "center" }} onClick={() => onNavigate(e.url)} title={e.url}>
                <span className="row-icon"><Globe size={14} /></span>
                <span className="row-main">
                  <span className="row-title">{e.title || domainOf(e.url)}</span>
                  <span className="row-sub">{e.url}</span>
                </span>
                <span className="row-time">{timeOf(e.visited_at)}</span>
              </button>
            </div>
          ))}
        </div>
      ))}
    </PanelShell>
  );
}

// ---------- Yer imleri ----------
export function BookmarksPanel({ onClose, onNavigate }: { onClose: () => void; onNavigate: (url: string) => void }) {
  const [query, setQuery] = useState("");
  const [items, setItems] = useState<BookmarkType[]>([]);
  const [loaded, setLoaded] = useState(false);

  const load = useCallback(() => {
    listBookmarks(query.trim() || undefined).then((rows) => {
      setItems(rows);
      setLoaded(true);
    });
  }, [query]);

  useEffect(() => {
    const t = window.setTimeout(load, 150);
    return () => window.clearTimeout(t);
  }, [load]);

  return (
    <PanelShell title="Yer İmleri" icon={Bookmark} onClose={onClose}>
      <div className="panel-search">
        <Search size={15} />
        <input
          autoFocus
          placeholder="Yer imlerinde ara…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          aria-label="Yer imlerinde ara"
        />
      </div>
      {loaded && items.length === 0 && (
        <div className="empty-state">
          <Bookmark size={38} strokeWidth={1.5} />
          <p>{query ? "Sonuç bulunamadı." : "Henüz yer imi yok — adres çubuğundaki yıldıza basın."}</p>
        </div>
      )}
      {items.map((b) => (
        <button key={b.id} className="row-item" onClick={() => onNavigate(b.url)} title={b.url}>
          <span className="row-icon" style={{ color: "var(--star)" }}><Bookmark size={14} fill="currentColor" /></span>
          <span className="row-main">
            <span className="row-title">{b.title || domainOf(b.url)}</span>
            <span className="row-sub">{b.url}</span>
          </span>
          <span
            className="row-action"
            role="button"
            aria-label="Yer imini sil"
            title="Yer imini sil"
            onClick={(e) => {
              e.stopPropagation();
              removeBookmark(b.url).then(load);
            }}
          >
            <Trash2 size={14} />
          </span>
        </button>
      ))}
    </PanelShell>
  );
}

// ---------- İndirmeler ----------
export function DownloadsPanel({ onClose }: { onClose: () => void }) {
  const [items, setItems] = useState<DownloadEntry[]>([]);

  const load = useCallback(() => {
    listDownloads().then(setItems);
  }, []);

  useEffect(() => {
    load();
    const unlisten = onDownloadsChanged(load);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [load]);

  return (
    <PanelShell
      title="İndirmeler"
      icon={Download}
      onClose={onClose}
      actions={
        <>
          <button className="btn small" onClick={() => openDownloadsFolder()}>
            <FolderOpen size={13} style={{ verticalAlign: "-2px", marginRight: 5 }} /> Klasörü Aç
          </button>
          <ArmedButton
            label="Tamamlananları Temizle"
            confirmLabel="Listeyi Sil"
            onConfirm={() => clearFinishedDownloads().then(load)}
          />
        </>
      }
    >
      {items.length === 0 && (
        <div className="empty-state">
          <Download size={38} strokeWidth={1.5} />
          <p>Henüz indirme yok. Bir dosya indirildiğinde burada ilerlemesini görürsünüz.</p>
        </div>
      )}
      {items.map((d) => {
        const pct = d.bytes_total > 0 ? Math.min(100, (d.bytes_received / d.bytes_total) * 100) : d.state === "completed" ? 100 : 0;
        return (
          <div key={d.id} className="row-item" style={{ cursor: "default" }}>
            <span className="row-icon">
              {d.state === "completed" ? <CheckCircle2 size={15} color="var(--ok)" /> : <FileClock size={15} />}
            </span>
            <span className="row-main">
              <span className="row-title">{d.file_name}</span>
              <span className="row-sub">
                {formatBytes(d.bytes_received)}
                {d.bytes_total > 0 ? ` / ${formatBytes(d.bytes_total)}` : ""} · {d.started_at.slice(0, 16)}
              </span>
              {d.state === "downloading" && (
                <span className="progress"><i style={{ width: `${pct}%` }} /></span>
              )}
            </span>
            <span className={`state-chip ${d.state}`}>
              {d.state === "downloading" ? `${Math.round(pct)}%` : d.state === "completed" ? "Tamam" : "Kesildi"}
            </span>
          </div>
        );
      })}
    </PanelShell>
  );
}

// ---------- Ayarlar ----------
export function SettingsPanel({
  snapshot,
  onClose,
  onSetTheme,
  onSetEngine,
  onToggleBlocker,
  onClearData,
  onSetSessionRestore,
}: {
  snapshot: Snapshot;
  onClose: () => void;
  onSetTheme: (theme: string) => void;
  onSetEngine: (engine: string) => void;
  onToggleBlocker: (enabled: boolean) => void;
  onClearData: () => void;
  onSetSessionRestore: (enabled: boolean) => void;
}) {
  return (
    <PanelShell title="Ayarlar" icon={Settings2} onClose={onClose}>
      <div className="card">
        <h2><Sun size={15} /> Genel</h2>
        <p>QuantumBrowser'ın görünümü ve varsayılan arama motoru.</p>
        <div className="card-row">
          <span style={{ fontWeight: 600 }}>Tema</span>
          <div className="seg" role="radiogroup" aria-label="Tema">
            {[
              { key: "light", label: "Açık", icon: Sun },
              { key: "dark", label: "Koyu", icon: Moon },
              { key: "system", label: "Sistem", icon: Monitor },
            ].map(({ key, label, icon: Icon }) => (
              <button
                key={key}
                role="radio"
                aria-checked={snapshot.theme === key}
                className={snapshot.theme === key ? "on" : ""}
                onClick={() => onSetTheme(key)}
              >
                <Icon size={13} /> {label}
              </button>
            ))}
          </div>
        </div>
        <div className="card-row" style={{ marginTop: 12 }}>
          <span style={{ fontWeight: 600 }}>Varsayılan arama motoru</span>
          <select
            className="styled"
            value={snapshot.search_engine}
            onChange={(e) => onSetEngine(e.target.value)}
            aria-label="Varsayılan arama motoru"
          >
            <option value="duckduckgo">DuckDuckGo</option>
            <option value="google">Google</option>
            <option value="bing">Bing</option>
            <option value="brave">Brave Search</option>
            <option value="ecosia">Ecosia</option>
          </select>
        </div>
      </div>

      <div className="card">
        <h2><ShieldCheck size={15} /> Gizlilik &amp; Engelleme</h2>
        <p>
          Yerleşik engelleme, bilinen reklam ve izleyici ağlarını (~110 alan adı) Chromium seviyesinde
          keser; ayrıca yaygın reklam çerçeveleri sayfada gizlenir. Engellenen sayı, sekme şeridindeki
          kalkan rozetinde görünür.
        </p>
        <div className="card-row">
          <span style={{ fontWeight: 600 }}>Reklam / izleyici engelleyici</span>
          <label className="switch" title="Reklam engelleyiciyi aç/kapat">
            <input
              type="checkbox"
              checked={snapshot.blocker_enabled}
              onChange={(e) => onToggleBlocker(e.target.checked)}
              aria-label="Reklam engelleyiciyi aç/kapat"
            />
            <span className="track" />
          </label>
        </div>
        <div className="card-row" style={{ marginTop: 12 }}>
          <span style={{ fontWeight: 600 }}>Kaldığım yerden aç</span>
          <label className="switch" title="Uygulama kapanırken açık sekmeleri aç">
            <input
              type="checkbox"
              checked={snapshot.session_restore}
              onChange={(e) => onSetSessionRestore(e.target.checked)}
              aria-label="Kaldığım yerden aç"
            />
            <span className="track" />
          </label>
        </div>
        <div className="card-row" style={{ marginTop: 12 }}>
          <span style={{ fontWeight: 600 }}>Geçmişi ve tamamlanan indirme listesini temizle</span>
          <ArmedButton
            label="Verileri Temizle"
            confirmLabel="Kesinlikle Temizle"
            onConfirm={onClearData}
          />
        </div>
        <div className="card-row" style={{ marginTop: 10 }}>
          <span className="row-sub" style={{ display: "flex", gap: 6, alignItems: "center" }}>
            <AlertTriangle size={12} /> Gizli sekmelerde geçmiş ve indirme kayıtları tutulmaz.
          </span>
        </div>
      </div>

      <div className="card">
        <h2><ShieldCheck size={15} /> Hesaplar &amp; Giriş</h2>
        <p>
          Google gibi servislerin oturum açma akışı bu profilden çerezlerle kalıcı tutulur —
          <strong>oturum açtığınız sürece tekrar giriş gerekmez.</strong> Gizli sekmede oturum saklanmaz.
        </p>
        <div className="card-row">
          <span style={{ fontWeight: 600 }}>Google'a giriş yap (WebView2)</span>
          <button className="btn" onClick={() => navigate("https://accounts.google.com")}>Aç</button>
        </div>
        <p style={{ margin: "8px 0 0", fontSize: 10.5 }}>
          Not: Bazı siteler gömülü tarayıcıları bilinçli olarak kısıtlar (403 / “disallowed_useragent”).
          Bu, sitenin tercihidir; çerez azaltma/QA'ları bizim tarayıcı katmanındadır.
        </p>
      </div>

      <div className="card">
        <h2><Info size={15} /> Hakkında</h2>
        <p style={{ marginBottom: 4 }}>
          <strong>QuantumBrowser</strong> v0.1.0 — WebView2 (Chromium) üzerine inşa edilen, Rust çekirdekli,
          Arc tarzı ferah arayüze sahip masaüstü tarayıcı.
        </p>
        <p style={{ margin: 0 }}>
          Tauri v2 · React · TypeScript · SQLite/FTS5 · Lucide ikonları · MIT Lisansı
        </p>
      </div>
    </PanelShell>
  );
}

// ---------- Profiller ----------
const PROFILE_COLORS = ["#7c6cf5", "#0ea5e9", "#10b981", "#f59e0b", "#ef4444", "#ec4899", "#14b8a6", "#8b5cf6"];

export function ProfilesPanel({
  snapshot,
  onClose,
  onRefresh,
  onSwitch,
}: {
  snapshot: Snapshot;
  onClose: () => void;
  onRefresh: () => void;
  onSwitch: (profileId: number) => void;
}) {
  const [newName, setNewName] = useState("");
  const [color, setColor] = useState(PROFILE_COLORS[0]);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editName, setEditName] = useState("");
  const [confirmDeleteId, setConfirmDeleteId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const initials = (name: string) => {
    const p = name.trim().split(/\s+/).filter(Boolean);
    return ((p[0]?.[0] ?? "?") + (p[1]?.[0] ?? "")).toUpperCase();
  };

  const startEdit = (id: number, current: string) => {
    setEditingId(id);
    setEditName(current);
    setError(null);
  };
  const saveEdit = (id: number) => {
    renameProfile(id, editName)
      .then(() => {
        setEditingId(null);
        onRefresh();
      })
      .catch((e) => setError(String(e)));
  };
  const removeProfile = (p: { id: number }) => {
    if (confirmDeleteId !== p.id) {
      setConfirmDeleteId(p.id);
      return;
    }
    deleteProfile(p.id)
      .then(() => {
        setConfirmDeleteId(null);
        onRefresh();
      })
      .catch((e) => {
        setConfirmDeleteId(null);
        setError(String(e));
      });
  };

  return (
    <PanelShell title="Profiller" icon={Users} onClose={onClose}>
      <p style={{ color: "var(--muted)", fontSize: 11.5, margin: "4px 2px 0" }}>
        Her profilin ayrı çerezleri ve oturumu vardır. Profil değiştirmek mevcut sekmeleri kapatır ve yeni
        boş oturum açar. Varsayılan profil ve aktif profil silinemez.
      </p>

      {error && (
        <div className="card" style={{ borderColor: "var(--danger)", color: "var(--danger)", padding: "10px 14px" }} role="alert">
          {error}
        </div>
      )}

      <div className="profile-grid">
        {snapshot.profiles.map((p) => {
          const current = p.id === snapshot.profile_id;
          const isEditing = editingId === p.id;
          return (
            <div key={p.id} className={`profile-card${current ? " current" : ""}`}>
              <span className="avatar-lg" style={{ background: p.color }}>{initials(p.name)}</span>

              {isEditing ? (
                <div className="rename-row">
                  <input
                    className="rename-input"
                    autoFocus
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") saveEdit(p.id);
                      if (e.key === "Escape") setEditingId(null);
                    }}
                    maxLength={40}
                    aria-label="Yeni profil adı"
                  />
                  <button className="icon-btn" title="Kaydet" aria-label="Kaydet" onClick={() => saveEdit(p.id)}>
                    <Check size={15} />
                  </button>
                </div>
              ) : (
                <span className="profile-card-name">{p.name}</span>
              )}

              {p.is_default && <span className="profile-default">Varsayılan</span>}

              {current ? (
                <span className="btn small" aria-disabled="true" style={{ opacity: 0.65, cursor: "default" }}>
                  <Check size={13} style={{ verticalAlign: "-2px", marginRight: 4 }} /> Mevcut Profil
                </span>
              ) : (
                <button className="btn small" onClick={() => onSwitch(p.id)}>Bu Profili Kullan</button>
              )}

              <div className="profile-actions">
                <button
                  className="icon-btn"
                  title="Yeniden adlandır"
                  aria-label="Yeniden adlandır"
                  onClick={() => (isEditing ? setEditingId(null) : startEdit(p.id, p.name))}
                >
                  <Pencil size={14} />
                </button>
                <button
                  className="icon-btn"
                  title={confirmDeleteId === p.id ? "Tekrar tıklayarak sil (geri alınamaz)" : "Profili sil"}
                  aria-label="Profili sil"
                  onClick={() => removeProfile(p)}
                  disabled={p.is_default || current}
                  style={p.is_default || current ? { opacity: 0.35, cursor: "default" } : undefined}
                >
                  {confirmDeleteId === p.id ? <Trash2 size={14} color="var(--danger)" /> : <Trash2 size={14} />}
                </button>
              </div>
            </div>
          );
        })}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h2><Plus size={15} /> Yeni Profil Ekle</h2>
        <div className="add-profile-row">
          <input
            placeholder="Profil adı (ör. İş, Aile, Oyun)…"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && newName.trim()) {
                addProfile(newName.trim(), color).then(() => {
                  setNewName("");
                  onRefresh();
                });
              }
            }}
            aria-label="Yeni profil adı"
            maxLength={40}
          />
          <div style={{ display: "flex", gap: 6, alignItems: "center" }} role="radiogroup" aria-label="Profil rengi">
            {PROFILE_COLORS.map((c) => (
              <button
                key={c}
                role="radio"
                aria-checked={color === c}
                className={`color-dot${color === c ? " on" : ""}`}
                style={{ background: c }}
                onClick={() => setColor(c)}
                aria-label={`Renk ${c}`}
              />
            ))}
          </div>
          <button
            className="btn"
            disabled={!newName.trim()}
            style={!newName.trim() ? { opacity: 0.45, cursor: "default" } : undefined}
            onClick={() =>
              addProfile(newName.trim(), color).then(() => {
                setNewName("");
                onRefresh();
              })
            }
          >
            Ekle
          </button>
        </div>
      </div>
    </PanelShell>
  );
}
