import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  message: string;
}

/**
 * Kabuk hatasına karşı dayanıklılık: React ağacı çökerse
 * beyaz ekran yerine bu ekran ve "Yenile" düğmesi görünür.
 */
export default class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, message: "" };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, message: String(error) };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("QuantumBrowser hata:", error, info);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="newtab" style={{ minHeight: "100vh", gap: 18 }}>
          <svg width="72" height="72" viewBox="0 0 32 32" fill="none" aria-hidden="true">
            <ellipse cx="16" cy="16" rx="12.5" ry="4.6" transform="rotate(-30 16 16)" stroke="#f87171" strokeWidth="2.2" strokeLinecap="round" />
            <ellipse cx="16" cy="16" rx="12.5" ry="4.6" transform="rotate(55 16 16)" stroke="#c084fc" strokeWidth="2.2" strokeLinecap="round" />
            <circle cx="16" cy="16" r="4.4" fill="#f87171" />
          </svg>
          <div className="greeting" style={{ marginTop: 0 }}>Bir şeyler ters gitti.</div>
          <div className="date-line">
            {this.state.message || "Bilinmeyen hata"}
          </div>
          <button className="btn-newtab" style={{ width: 260 }} onClick={() => window.location.reload()}>
            QuantumBrowser'ı Yeniden Yükle
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
