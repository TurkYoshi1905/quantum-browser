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
      <img className="logo-mark" src="/quantum-browser.png" alt="QuantumBrowser" width={18} height={18} />
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
