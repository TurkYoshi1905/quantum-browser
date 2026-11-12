import { useEffect, useState } from "react";
import { CheckCircle2, X, Download } from "lucide-react";
import * as api from "../api";
import { applyRootTheme } from "../utils/theme";

// Rust `show_toast` bu webview'ı sayfanın en üstünde (kabuk penceresi içinde) gösterir.
// 4.2 saniye sonra kendi kendini kapatır.
export default function Toast() {
  const [text, setText] = useState("İndirme tamamlandı");

  useEffect(() => {
    api.getSnapshot()
      .then((s) => applyRootTheme(s.theme || "light", s.accent || "mor"))
      .catch(() => {});
    const un = api.onToastOpen((p) => setText(p.text));
    const timer = window.setTimeout(() => api.closeToast().catch(() => {}), 4200);
    return () => {
      un.then((f) => f());
      window.clearTimeout(timer);
    };
  }, []);

  return (
    <main className="toast-hud" role="status" aria-live="polite">
      <span className="toast-hud-icon"><Download size={16} strokeWidth={2} /></span>
      <div className="toast-hud-body">
        <span className="toast-hud-title">
          <CheckCircle2 size={12} strokeWidth={2.4} style={{ color: "var(--ok)" }} />
          İndirme tamamlandı
        </span>
        <span className="toast-hud-text" title={text}>{text}</span>
      </div>
      <button className="toast-hud-close" aria-label="Kapat" onClick={() => api.closeToast().catch(() => {})}>
        <X size={14} strokeWidth={2} />
      </button>
    </main>
  );
}
