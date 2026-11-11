// Apply tema + accent değişkenlerini document root'a uygular
export function applyRootTheme(theme: string, accent: string = "mor") {
  const dark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
  document.documentElement.dataset.accent = accent;
}
