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
        <svg className="logo-mark" viewBox="0 0 32 32" fill="none">
          <defs>
            <linearGradient id="qbA" x1="0" y1="0" x2="1" y2="1">
              <stop offset="0" stopColor="#22d3ee" />
              <stop offset="1" stopColor="#818cf8" />
            </linearGradient>
            <linearGradient id="qbB" x1="1" y1="0" x2="0" y2="1">
              <stop offset="0" stopColor="#c084fc" />
              <stop offset="1" stopColor="#8b5cf6" />
            </linearGradient>
          </defs>
          <ellipse cx="16" cy="16" rx="12.5" ry="4.6" transform="rotate(-30 16 16)" stroke="url(#qbA)" strokeWidth="2.6" strokeLinecap="round" />
          <ellipse cx="16" cy="16" rx="12.5" ry="4.6" transform="rotate(55 16 16)" stroke="url(#qbB)" strokeWidth="2.6" strokeLinecap="round" />
          <circle cx="16" cy="16" r="4.2" fill="#67e8f9" />
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
