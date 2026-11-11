import { useEffect, useState } from "react";
import { MoonStar, Search, Sun, Sunrise, Sparkles } from "lucide-react";
import { getSnapshot } from "../api";
import { applyRootTheme } from "../utils/theme";

const SEARCH_TEMPLATES: Record<string, string> = {
  duckduckgo: "https://duckduckgo.com/?q=",
  google: "https://www.google.com/search?q=",
  bing: "https://www.bing.com/search?q=",
  brave: "https://search.brave.com/search?q=",
  ecosia: "https://www.ecosia.org/search?q=",
};
const ENGINE_NAMES: Record<string, string> = {
  duckduckgo: "DuckDuckGo", google: "Google", bing: "Bing",
  brave: "Brave Search", ecosia: "Ecosia",
};
const QUICK_LINKS = [
  { name: "YouTube", url: "https://www.youtube.com", letter: "Y", color: "#e11d48" },
  { name: "X (Twitter)", url: "https://x.com", letter: "X", color: "#111318" },
  { name: "GitHub", url: "https://github.com", letter: "G", color: "#24292f" },
  { name: "Reddit", url: "https://www.reddit.com", letter: "R", color: "#ff4500" },
  { name: "Gmail", url: "https://mail.google.com", letter: "M", color: "#ea4335" },
  { name: "Vikipedi", url: "https://tr.wikipedia.org", letter: "V", color: "#3b3f46" },
  { name: "Hacker News", url: "https://news.ycombinator.com", letter: "H", color: "#ff6600" },
  { name: "Çeviri", url: "https://translate.google.com", letter: "Ç", color: "#4285f4" },
];

export default function NewTab() {
  const [now, setNow] = useState(() => new Date());
  const [engine, setEngine] = useState("duckduckgo");
  const [theme, setTheme] = useState("light");
  const [accent, setAccent] = useState("mor");
  const [query, setQuery] = useState("");
  const [profileColor, setProfileColor] = useState("#7c6cf5");

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1000);
    getSnapshot()
      .then((s) => {
        setEngine(s.search_engine || "duckduckgo");
        setTheme(s.theme || "light");
        setAccent(s.accent || "mor");
        setProfileColor(s.profile_color || "#7c6cf5");
        applyRootTheme(s.theme || "light", s.accent || "mor");
      })
      .catch(() => {});
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    const apply = () => {
      const dark =
        theme === "dark" || (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
      document.documentElement.dataset.theme = dark ? "dark" : "light";
      document.documentElement.dataset.accent = accent;
    };
    apply();
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    mq.addEventListener?.("change", apply);
    return () => mq.removeEventListener?.("change", apply);
  }, [theme, accent]);

  const hour = now.getHours();
  const greeting = hour < 6 ? "İyi geceler" : hour < 12 ? "Günaydın" : hour < 18 ? "İyi günler" : "İyi akşamlar";
  const GreetingIcon = hour < 6 ? MoonStar : hour < 12 ? Sunrise : hour < 18 ? Sun : MoonStar;
  const timeText = now.toLocaleTimeString("tr-TR", { hour: "2-digit", minute: "2-digit" });
  const dateText = now.toLocaleDateString("tr-TR", { weekday: "long", day: "numeric", month: "long", year: "numeric" });

  const submit = () => {
    const q = query.trim();
    if (!q) return;
    let target: string;
    if (/^https?:\/\//i.test(q)) target = q;
    else if (/^[^\s.]+\.[^\s]{2,}$/.test(q) && !q.includes(" ")) target = `https://${q}`;
    else target = (SEARCH_TEMPLATES[engine] ?? SEARCH_TEMPLATES.duckduckgo) + encodeURIComponent(q);
    window.location.assign(target);
  };

  return (
    <main className="nt-hero">
      <div className="nt-ambient nt-ambient-a" />
      <div className="nt-ambient nt-ambient-b" />
      <div className="nt-ambient nt-ambient-c" />

      <div className="newtab-wordmark" style={{ top: 26, left: 30, position: "absolute" }}>
        <img className="wordmark-img" src="/quantum-browser.png" alt="" width={28} height={28} />
        <span>QuantumBrowser</span>
      </div>

      <div className="nt-time-block">
        <div className="clock" aria-label={timeText}>{timeText}</div>
        <div className="date-line">{dateText}</div>
        <div className="greeting">
          <GreetingIcon size={19} strokeWidth={1.8} />
          {greeting}! Nereye gidelim?
        </div>
      </div>

      <form className="omni-big" onSubmit={(e) => { e.preventDefault(); submit(); }} role="search">
        <Search size={19} strokeWidth={2} aria-hidden="true" />
        <input
          autoFocus
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={`${ENGINE_NAMES[engine] ?? "DuckDuckGo"}'da ara veya bir adres yaz…`}
          aria-label="Arama veya adres"
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
        />
      </form>

      <div className="quick-grid" aria-label="Hızlı erişim">
        {QUICK_LINKS.map((link) => (
          <a key={link.url} className="quick-item" href={link.url}>
            <span className="quick-tile" style={{ background: link.color }}>{link.letter}</span>
            <span className="quick-name">{link.name}</span>
          </a>
        ))}
      </div>

      <div className="nt-tip">
        <Sparkles size={11} /> F1 = Bir şey arayın · Ctrl+K = Adres çubuğu · Ctrl+T = Yeni sekme
      </div>

      <div className="newtab-foot">
        <span>Reklam engelleyici yerleşik</span>
        <span>· Telemetri yok</span>
        <span>· Quantum Şifre Kasası</span>
      </div>
    </main>
  );
}
