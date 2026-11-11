import { ShieldAlert, XOctagon, ArrowRight } from "lucide-react";
import { useEffect } from "react";
import * as api from "../api";
import { applyRootTheme } from "../utils/theme";

// Rust `navigate_active` http:// adreslerini "quantum://blocked?url=..."e çevirir.
function blockedUrl(): string {
  const hash = window.location.hash || "";
  const query = hash.split("?")[1] || "";
  const params = new URLSearchParams(query);
  return params.get("url") || "";
}

export default function Blocked() {
  const target = blockedUrl();

  useEffect(() => {
    api.getSnapshot()
      .then((s) => applyRootTheme(s.theme || "light", s.accent || "mor"))
      .catch(() => {});
  }, []);

  return (
    <main className="blocked-page">
      <div className="blocked-card">
        <div className="blocked-icon" aria-hidden="true">
          <XOctagon size={56} strokeWidth={1.6} />
        </div>
        <h1 className="blocked-title">Bu site güvenli değil</h1>
        <p className="blocked-sub">
          <strong>{target || "Bilinmeyen adres"}</strong> HTTPS kullanmıyor (HTTP).
          Bağlantı şifreli olmadığı için bilgileriniz (şifreniz, kartınız, mesajlarınız)
          ağdaki herkes tarafından okunabilir.
        </p>

        <div className="blocked-box">
          <ShieldAlert size={16} strokeWidth={2} />
          <div>
            <div className="blocked-box-title">QuantumBrowser neden engelledi?</div>
            <div className="blocked-box-text">
              HTTPS (TLS) zorunluluğu açık. Sertifika doğrulanmamış bir bağlantıya
              giriş yapmanızı önlemek istiyoruz.
            </div>
          </div>
        </div>

        <div className="blocked-actions">
          <button className="blocked-primary" onClick={() => window.history.back()}>
            <ArrowRight size={16} style={{ transform: "rotate(180deg)" }} /> Güvenli yere dön
          </button>
          {target && (
            <button className="blocked-secondary"
              onClick={() => api.navigate(target + (target.includes("?") ? "&" : "?") + "qb_bypass=1")}
            >
              Yine de devam et (tavsiye edilmez)
            </button>
          )}
        </div>
      </div>
    </main>
  );
}
