import { useEffect, useRef, useState } from "react";
import {
  ArrowLeft,
  ArrowRight,
  Bookmark,
  BookmarkPlus,
  EyeOff,
  FileClock,
  FolderDown,
  Lock,
  MoreVertical,
  Plus,
  RotateCw,
  Settings,
  Shield,
  ShieldCheck,
  ShieldOff,
  Star,
  TriangleAlert,
} from "lucide-react";
import type { Panel } from "./Sidebar";

interface Props {
  url: string;
  isIncognito: boolean;
  isNewTab: boolean;
  blockedCount: number;
  bookmarked: boolean;
  blockerEnabled: boolean;
  inputRef: React.RefObject<HTMLInputElement>;
  onNavigate: (input: string) => void;
  onBack: () => void;
  onForward: () => void;
  onReload: () => void;
  onToggleBookmark: () => void;
  onToggleBlocker: () => void;
  onNewTab: () => void;
  onNewIncognito: () => void;
  onOpenPanel: (panel: Panel) => void;
  onMenuChange?: (open: boolean) => void;
}

export default function AddressBar(props: Props) {
  const { url, inputRef, onMenuChange } = props;
  const [value, setValue] = useState(url);
  const [menuOpen, setMenuOpen] = useState(false);
  const [spinning, setSpinning] = useState(false);
  const [starPop, setStarPop] = useState(false);
  const [chipPulse, setChipPulse] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const prevBlocked = useRef(props.blockedCount);

  // Sekme URL'i değiştiğinde kutuyu güncelle
  useEffect(() => {
    setValue(props.isNewTab ? "" : url);
  }, [url, props.isNewTab]);

  // Menü açık/kapalı bilgisini üst yönetime bildir (çocuk webview'ı küçültmek için)
  useEffect(() => {
    onMenuChange?.(menuOpen);
  }, [menuOpen, onMenuChange]);

  // Menü dışı tıklamada kapat
  useEffect(() => {
    if (!menuOpen) return;
    const close = (e: MouseEvent) => {
      if (!menuRef.current?.contains(e.target as Node)) setMenuOpen(false);
    };
    window.addEventListener("click", close);
    return () => window.removeEventListener("click", close);
  }, [menuOpen]);

  // Engelleme sayacı arttığında kalkan rozetine pulse animasyonu
  useEffect(() => {
    if (props.blockedCount > prevBlocked.current) {
      setChipPulse(true);
      const t = window.setTimeout(() => setChipPulse(false), 380);
      prevBlocked.current = props.blockedCount;
      return () => window.clearTimeout(t);
    }
    prevBlocked.current = props.blockedCount;
  }, [props.blockedCount]);

  const secure = url.startsWith("https://");
  const insecure = url.startsWith("http://");

  const submit = () => {
    props.onNavigate(value.trim());
    inputRef.current?.blur();
  };

  return (
    <div className="addressbar">
      <button className="nav-btn" title="Geri (Alt+Sol)" aria-label="Geri" onClick={props.onBack}>
        <ArrowLeft size={16} strokeWidth={2} />
      </button>
      <button className="nav-btn" title="İleri (Alt+Sağ)" aria-label="İleri" onClick={props.onForward}>
        <ArrowRight size={16} strokeWidth={2} />
      </button>
      <button
        className={`nav-btn${spinning ? " spinning" : ""}`}
        title="Yenile (F5)"
        aria-label="Yenile"
        onClick={() => {
          props.onReload();
          setSpinning(true);
          window.setTimeout(() => setSpinning(false), 650);
        }}
      >
        <RotateCw size={15} strokeWidth={2} />
      </button>

      <div className="omnibox" role="combobox" aria-expanded="false" aria-label="Adres çubuğu">
        {props.isIncognito ? (
          <span className="chip-lock incognito" title="Gizli mod — geçmiş kaydedilmez">
            <EyeOff size={11} strokeWidth={2.4} /> Gizli
          </span>
        ) : secure ? (
          <span className="chip-lock secure" title="Güvenli bağlantı (HTTPS)">
            <Lock size={11} strokeWidth={2.4} /> Güvenli
          </span>
        ) : insecure ? (
          <span className="chip-lock" title="Güvenli olmayan bağlantı (HTTP)" style={{ color: "var(--star)" }}>
            <TriangleAlert size={11} strokeWidth={2.4} /> HTTP
          </span>
        ) : (
          <span className="chip-lock" title="Yeni sekme">
            <Lock size={11} strokeWidth={2.4} />
          </span>
        )}

        <input
          ref={inputRef}
          type="text"
          inputMode="url"
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          placeholder="Adres veya arama…"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
            else if (e.key === "Escape") {
              setValue(props.isNewTab ? "" : url);
              inputRef.current?.blur();
            }
          }}
          onFocus={(e) => {
            if (value) e.target.select();
          }}
          aria-label="Adres veya arama"
        />

        <button
          className={`star-btn${props.bookmarked ? " active" : ""}${starPop ? " star-pop" : ""}`}
          onClick={() => {
            props.onToggleBookmark();
            setStarPop(true);
            window.setTimeout(() => setStarPop(false), 450);
          }}
          title={props.bookmarked ? "Yer imini kaldır" : "Yer imlerine ekle"}
          aria-label={props.bookmarked ? "Yer imini kaldır" : "Yer imlerine ekle"}
          aria-pressed={props.bookmarked}
        >
          <Star size={16} strokeWidth={2} fill={props.bookmarked ? "currentColor" : "none"} />
        </button>

        <span className="omni-sep" aria-hidden="true" />

        <button
          className={`chip-block${props.blockerEnabled ? "" : " off"}${chipPulse ? " chip-pulse" : ""}`}
          onClick={props.onToggleBlocker}
          title={props.blockerEnabled ? "Reklam engelleyici açık — kapatmak için tıklayın" : "Reklam engelleyici kapalı — açmak için tıklayın"}
          aria-label="Reklam engelleyiciyi aç/kapat"
        >
          <ShieldCheck size={11} strokeWidth={2.4} />
          {props.blockedCount > 0 ? props.blockedCount : props.blockerEnabled ? "Açık" : "Kapalı"}
        </button>
      </div>

      <div style={{ position: "relative" }} ref={menuRef}>
        <button
          className="nav-btn"
          title="Menü"
          aria-label="Menü"
          aria-expanded={menuOpen}
          onClick={() => setMenuOpen((v) => !v)}
        >
          <MoreVertical size={17} strokeWidth={2} style={{ color: "var(--text)" }} />
        </button>

        {menuOpen && (
          <div className="dropdown" role="menu" aria-label="QuantumBrowser menüsü">
            <button className="dropdown-item" onClick={() => { setMenuOpen(false); props.onNewTab(); }}>
              <Plus size={15} /> Yeni Sekme <span className="kbd">Ctrl T</span>
            </button>
            <button className="dropdown-item" onClick={() => { setMenuOpen(false); props.onNewIncognito(); }}>
              <EyeOff size={15} /> Gizli Sekme <span className="kbd">Ctrl ⇧ N</span>
            </button>
            <div className="dropdown-sep" />
            <button className="dropdown-item" onClick={() => { setMenuOpen(false); props.onOpenPanel("bookmarks"); }}>
              <Bookmark size={15} /> Yer İmleri <span className="kbd">Ctrl B</span>
            </button>
            <button className="dropdown-item" onClick={() => { setMenuOpen(false); props.onOpenPanel("history"); }}>
              <FileClock size={15} /> Geçmiş <span className="kbd">Ctrl ⇧ Y</span>
            </button>
            <button className="dropdown-item" onClick={() => { setMenuOpen(false); props.onOpenPanel("downloads"); }}>
              <FolderDown size={15} /> İndirmeler <span className="kbd">Ctrl J</span>
            </button>
            <div className="dropdown-sep" />
            <button className="dropdown-item" onClick={() => { setMenuOpen(false); props.onOpenPanel("settings"); }}>
              <Settings size={15} /> Ayarlar
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
