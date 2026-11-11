import { useMemo, useEffect, useState } from "react";
import { applyRootTheme } from "../utils/theme";
import {
  KeyRound,
  Lock,
  Unlock,
  Plus,
  Copy,
  Eye,
  EyeOff,
  Trash2,
  Search,
  ShieldCheck,
  Wand2,
  Download,
  Upload,
  X,
  Wallet,
} from "lucide-react";
import * as api from "../api";

interface VaultItem {
  id: number;
  host: string;
  username: string;
  created_at: string;
  updated_at: string;
}

export default function Vault() {
  const [status, setStatus] = useState<{ hasMasterPassword: boolean; unlocked: boolean } | null>(null);
  const [master, setMaster] = useState("");
  const [items, setItems] = useState<VaultItem[]>([]);
  const [query, setQuery] = useState("");
  const [revealId, setRevealId] = useState<number | null>(null);
  const [revealText, setRevealText] = useState("");
  const [formOpen, setFormOpen] = useState(false);
  const [host, setHost] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [note, setNote] = useState("");
  const [toast, setToast] = useState("");

  const refresh = async () => {
    const s = await api.vaultStatus();
    setStatus(s);
    if (s.unlocked) {
      try { setItems(await api.vaultList()); } catch { setItems([]); }
    }
  };

  useEffect(() => {
    api.getSnapshot()
      .then((s) => applyRootTheme(s.theme || "light", s.accent || "mor"))
      .catch(() => {});
    refresh();
  }, []);

  const filtered = useMemo(
    () => items.filter((i) =>
      !query || i.host.toLowerCase().includes(query.toLowerCase()) || i.username.toLowerCase().includes(query.toLowerCase())
    ),
    [items, query]
  );

  const flash = (msg: string) => {
    setToast(msg);
    window.setTimeout(() => setToast(""), 2200);
  };

  const setMasterPassword = async () => {
    if (master.length < 4) return flash("En az 4 karakter");
    try {
      await api.vaultSetMasterPassword(master);
      setMaster("");
      flash("Ana şifre belirlendi ✓");
      refresh();
    } catch (e) {
      flash(String(e));
    }
  };

  const unlock = async () => {
    try {
      const ok = await api.vaultUnlock(master);
      setMaster("");
      if (!ok) flash("Yanlış ana şifre");
      else flash("Kasa açıldı");
      refresh();
    } catch (e) {
      flash(String(e));
    }
  };

  const lockNow = async () => {
    await api.vaultLock();
    refresh();
  };

  const addItem = async () => {
    if (!host.trim() || !password) return flash("Site ve şifre zorunlu");
    try {
      await api.vaultAdd(host.trim(), username.trim(), password, note.trim() || undefined);
      setHost(""); setUsername(""); setPassword(""); setNote(""); setFormOpen(false);
      flash("Kayıt eklendi ✓");
      refresh();
    } catch (e) {
      flash(String(e));
    }
  };

  const removeItem = async (id: number) => {
    try {
      await api.vaultDelete(id);
      flash("Silindi");
      refresh();
    } catch (e) {
      flash(String(e));
    }
  };

  const reveal = async (id: number) => {
    if (revealId === id) { setRevealId(null); setRevealText(""); return; }
    try {
      const p = await api.vaultGetPassword(id);
      setRevealId(id);
      setRevealText(p);
    } catch (e) {
      flash(String(e));
    }
  };

  const copyPassword = async (id: number) => {
    try {
      const p = await api.vaultGetPassword(id);
      await navigator.clipboard.writeText(p);
      flash("Panoya kopyalandı (30 sn içinde temizlenir)");
      window.setTimeout(() => navigator.clipboard.writeText(""), 30_000);
    } catch (e) {
      flash(String(e));
    }
  };

  const genPassword = async () => {
    try {
      const p = await api.vaultGeneratePassword();
      setPassword(p);
    } catch {
      /* sessiz */
    }
  };

  const exportVault = async () => {
    try {
      const text = await api.vaultExport();
      const blob = new Blob([text], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "quantum-vault.qbvault";
      a.click();
      URL.revokeObjectURL(url);
      flash("Yedek indirildi ✓");
    } catch (e) {
      flash(String(e));
    }
  };

  const importVault = async (file: File | undefined) => {
    if (!file) return;
    const text = await file.text();
    try {
      const n = await api.vaultImport(text);
      flash(`${n} kayıt içe aktarıldı`);
      refresh();
    } catch (e) {
      flash(String(e));
    }
  };

  if (!status) {
    return (
      <main className="vault-page vault-centered">
        <div className="vault-loading">Yükleniyor…</div>
      </main>
    );
  }

  // ---------- Kilit ekranı ----------
  if (!status.unlocked) {
    return (
      <main className="vault-page vault-centered">
        <div className="vault-lock-card">
          <div className="vault-lock-icon"><KeyRound size={44} strokeWidth={1.6} /></div>
          <h1 className="vault-lock-title">Quantum Şifre Kasası</h1>
          <p className="vault-lock-sub">
            {status.hasMasterPassword
              ? "Kasanızı açmak için ana şifrenizi girin."
              : "Kasanızı ilk kez kuruyoruz — bir ana şifre seçin. Bu şifreyi unutursanız kayıtlar kurtarılamaz."}
          </p>
          <input
            type="password"
            className="vault-input"
            autoFocus
            placeholder={status.hasMasterPassword ? "Ana şifre" : "Yeni ana şifre"}
            value={master}
            onChange={(e) => setMaster(e.target.value)}
            onKeyDown={(e) => (e.key === "Enter" ? (status.hasMasterPassword ? unlock() : setMasterPassword()) : undefined)}
            aria-label="Ana şifre"
          />
          <button className="vault-primary-btn" onClick={status.hasMasterPassword ? unlock : setMasterPassword}>
            <Unlock size={16} /> {status.hasMasterPassword ? "Kasayı Aç" : "Kasayı Oluştur"}
          </button>
          {toast && <div className="vault-toast">{toast}</div>}
        </div>
      </main>
    );
  }

  // ---------- Kasa açık: liste ----------
  return (
    <main className="vault-page">
      <header className="vault-head">
        <div className="vault-head-left">
          <Wallet size={20} strokeWidth={1.8} />
          <div>
            <h1 className="vault-title">Quantum Şifre Kasası</h1>
            <p className="vault-sub">{items.length} kayıtlı giriş · cihazınızda şifreli</p>
          </div>
        </div>
        <div className="vault-head-actions">
          <button className="vault-icon-btn" title="Dışa aktar" onClick={exportVault} aria-label="Dışa aktar">
            <Download size={17} />
          </button>
          <label className="vault-icon-btn" title="İçe aktar" aria-label="İçe aktar">
            <Upload size={17} />
            <input type="file" accept=".qbvault,.json,application/json" hidden onChange={(e) => importVault(e.target.files?.[0])} />
          </label>
          <button className="vault-icon-btn" title="Kilitle" onClick={lockNow} aria-label="Kasayı kilitle">
            <Lock size={17} />
          </button>
        </div>
      </header>

      <div className="vault-toolbar">
        <div className="vault-search">
          <Search size={14} />
          <input
            placeholder="Site veya kullanıcı adında ara…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            aria-label="Ara"
          />
        </div>
        <button className="vault-add-btn" onClick={() => setFormOpen((v) => !v)}>
          <Plus size={16} /> Kayıt Ekle
        </button>
      </div>

      {formOpen && (
        <div className="vault-form">
          <div className="vault-form-row">
            <input className="vault-input" placeholder="Site (örn. github.com)" value={host} onChange={(e) => setHost(e.target.value)} />
            <input className="vault-input" placeholder="Kullanıcı adı" value={username} onChange={(e) => setUsername(e.target.value)} />
          </div>
          <div className="vault-form-row">
            <input className="vault-input" placeholder="Şifre" value={password} onChange={(e) => setPassword(e.target.value)} />
            <button className="vault-icon-btn" onClick={genPassword} title="Güçlü şifre üret" aria-label="Şifre üret">
              <Wand2 size={17} />
            </button>
          </div>
          <input className="vault-input" placeholder="Not (opsiyonel)" value={note} onChange={(e) => setNote(e.target.value)} />
          <div className="vault-form-actions">
            <button className="vault-secondary-btn" onClick={() => setFormOpen(false)}>Vazgeç</button>
            <button className="vault-primary-btn" onClick={addItem}><Plus size={15} /> Kaydet</button>
          </div>
        </div>
      )}

      <section className="vault-list">
        {filtered.length === 0 && (
          <div className="vault-empty">
            <ShieldCheck size={36} strokeWidth={1.4} />
            <p>{items.length === 0 ? "Henüz kayıt yok — ilk şifrenizi kaydedin." : "Sonuç bulunamadı."}</p>
          </div>
        )}
        {filtered.map((item) => (
          <div key={item.id} className="vault-item">
            <div className="vault-item-main">
              <div className="vault-item-host">{item.host}</div>
              <div className="vault-item-user">{item.username || "—"}</div>
              {revealId === item.id && <div className="vault-item-secret">{revealText}</div>}
            </div>
            <div className="vault-item-actions">
              <button className="vault-icon-btn" title={revealId === item.id ? "Gizle" : "Göster"} onClick={() => reveal(item.id)} aria-label="Şifre göster/gizle">
                {revealId === item.id ? <EyeOff size={16} /> : <Eye size={16} />}
              </button>
              <button className="vault-icon-btn" title="Kopyala" onClick={() => copyPassword(item.id)} aria-label="Kopyala">
                <Copy size={16} />
              </button>
              <button className="vault-icon-btn danger" title="Sil" onClick={() => removeItem(item.id)} aria-label="Sil">
                <Trash2 size={16} />
              </button>
            </div>
          </div>
        ))}
      </section>

      {toast && <div className="vault-toast">{toast}</div>}
    </main>
  );
}
