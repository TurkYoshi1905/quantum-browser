import { useEffect, useRef, useState } from "react";
import {
  ArrowLeft,
  ArrowRight,
  Atom,
  EyeOff,
  Lock,
  MoreVertical,
  RotateCw,
  ShieldCheck,
  Star,
  TriangleAlert,
} from "lucide-react";
import * as api from "../api";
import type { Panel } from "./Sidebar";

interface Props {
  url: string;
  isIncognito: boolean;
  isNewTab: boolean;
  blockedCount: number;
  bookmarked: boolean;
  blockerEnabled: boolean;
  internalKind?: string | null;
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
}

export default function AddressBar(props: Props) {
  const { url, inputRef } = props;
  const [value, setValue] = useState(url);
  const [spinning, setSpinning] = useState(false);
  const [starPop, setStarPop] = useState(false);
  const [chipPulse, setChipPulse] = useState(false);
  const prevBlocked = useRef(props.blockedCount);

  // Sekme URL'i değiştiğinde kutuyu güncelle
  useEffect(() => {
    setValue(props.isNewTab ? "" : url);
  }, [url, props.isNewTab]);

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

  // Menüyü kabuk penceresi içindeki çocuk webview'da aç (butona göre viewport koordinatı).
  const openMainMenu = (e: React.MouseEvent) => {
    const btn = e.currentTarget as HTMLElement;
    const rect = btn.getBoundingClientRect();
    const width = 264;
    const estimated = 344;
    let x = rect.right - width; // sağ kenarı butona hizala
    if (x < 8) x = 8;
    let y = rect.bottom + 6;
    if (y + estimated > window.innerHeight - 12) {
      y = rect.top - estimated - 6; // altta yer yoksa üste aç
      if (y < 8) y = window.innerHeight - estimated - 12;
    }
    api.openMenuPopup("main", x, y);
  };

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
        {props.internalKind ? (
          <span className="chip-lock" title="QuantumBrowser iç sayfası" style={{ color: "var(--accent-2)" }}>
            <Atom size={11} strokeWidth={2.2} /> Quantum
          </span>
        ) : props.isIncognito ? (
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

        {!props.internalKind && (
          <>
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
          </>
        )}
      </div>

      <div>
        <button
          className="nav-btn"
          data-menu-anchor
          title="Menü"
          aria-label="Menü"
          onClick={(e) => {
            void openMainMenu(e);
          }}
        >
          <MoreVertical size={17} strokeWidth={2} style={{ color: "var(--text)" }} />
        </button>
      </div>
    </div>
  );
}
