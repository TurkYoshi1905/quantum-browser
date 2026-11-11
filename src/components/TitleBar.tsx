import { Minus, Square, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface Props {
  profileName: string;
  title: string;
}

/** QuantumBrowser özel başlık çubuğu (pencere sürükleme bölgesi + pencere düğmeleri) */
export default function TitleBar({ profileName, title }: Props) {
  const appWindow = getCurrentWindow();

  return (
    <header className="titlebar" data-tauri-drag-region="">
      <div className="logo" aria-hidden="true">
        <svg className="logo-mark" viewBox="0 0 32 32" fill="none" aria-hidden="true">
          <defs>
            <linearGradient id="qbStroke" x1="0" y1="0" x2="1" y2="1">
              <stop offset="0" stopColor="#22d3ee" />
              <stop offset="0.5" stopColor="#818cf8" />
              <stop offset="1" stopColor="#a78bfa" />
            </linearGradient>
          </defs>
          <circle cx="13" cy="13" r="10" stroke="url(#qbStroke)" strokeWidth="3.4" />
          <path d="M19.5 19.5 L28 28" stroke="url(#qbStroke)" strokeWidth="3.4" strokeLinecap="round" />
        </svg>
        <span>QuantumBrowser</span>
        <span className="profile-chip">{profileName}</span>
      </div>

      {/* Sürükleme bölgesi — çift tıklamayla büyüt/küçült */}
      <div
        className="titlebar-drag"
        data-tauri-drag-region=""
        onDoubleClick={() => appWindow.toggleMaximize()}
      >
        <span className="titlebar-title">{title}</span>
      </div>

      <div className="win-controls">
        <button className="win-btn" title="Küçült" aria-label="Küçült" onClick={() => appWindow.minimize()}>
          <Minus size={15} strokeWidth={1.8} />
        </button>
        <button className="win-btn" title="Büyüt / Geri Yükle" aria-label="Büyüt" onClick={() => appWindow.toggleMaximize()}>
          <Square size={12} strokeWidth={1.8} />
        </button>
        <button className="win-btn close" title="Kapat" aria-label="Kapat" onClick={() => appWindow.close()}>
          <X size={15} strokeWidth={1.8} />
        </button>
      </div>
    </header>
  );
}
