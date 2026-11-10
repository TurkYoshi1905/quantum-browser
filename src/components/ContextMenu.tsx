import { useEffect, useRef } from "react";
import type { LucideIcon } from "lucide-react";

export interface MenuItem {
  label: string;
  icon?: LucideIcon;
  danger?: boolean;
  onSelect: () => void;
}
export type MenuEntry = MenuItem | "separator";

interface Props {
  x: number;
  y: number;
  entries: MenuEntry[];
  onClose: () => void;
}

/** Sağ tık bağlam menüsü — Esc/dışarı tık ile kapanır */
export default function ContextMenu({ x, y, entries, onClose }: Props) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const close = () => onClose();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("click", close);
    window.addEventListener("contextmenu", close);
    window.addEventListener("keydown", onKey);
    window.addEventListener("blur", close);
    return () => {
      window.removeEventListener("click", close);
      window.removeEventListener("contextmenu", close);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("blur", close);
    };
  }, [onClose]);

  // Ekranın dışına taşmasın
  const width = 230;
  const height = entries.length * 34 + 16;
  const left = Math.min(x, window.innerWidth - width - 8);
  const top = Math.min(y, window.innerHeight - height - 8);

  return (
    <div ref={ref} className="context-menu" style={{ left, top }} role="menu" aria-label="Sekme menüsü">
      {entries.map((entry, i) => {
        if (entry === "separator") return <div key={i} className="dropdown-sep" />;
        const Icon = entry.icon;
        return (
          <button
            key={i}
            role="menuitem"
            className="dropdown-item"
            style={entry.danger ? { color: "var(--danger)" } : undefined}
            onClick={(e) => {
              e.stopPropagation();
              onClose();
              entry.onSelect();
            }}
          >
            {Icon ? <Icon size={15} strokeWidth={2} /> : null}
            <span>{entry.label}</span>
          </button>
        );
      })}
    </div>
  );
}
