// QuantumBrowser — her zaman üstte, dekorsuz, şeffaf popover içeriği.
// Kabuk "menu-open" olayı ile menü tipini ve (tab menüsünde) sekme kimliğini yayınla.
import { useEffect, useState } from "react";
import type { Snapshot } from "../types";
import * as api from "../api";
import {
  Atom,
  Bookmark,
  Download,
  EyeOff,
  FileClock,
  FolderPlus,
  History,
  Plus,
  Settings,
  Copy,
  Pin,
  PinOff,
  Trash2,
  X,
  FolderMinus,
} from "lucide-react";

interface MenuState {
  menu: string;
  tabId: string | null;
}

export default function MenuPopup() {
  const [state, setState] = useState<MenuState | null>(null);
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);

  useEffect(() => {
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
    const un = api.onMenuOpen((p) => setState(p));
    // Yarışı ortadan kaldır: mount anında bekleyen menüyü çek
    api.getPendingMenu().then((p) => {
      if (p) setState(p);
    }).catch(() => {});
    api.getSnapshot().then(setSnapshot).catch(() => {});
    return () => {
      un.then((f) => f());
    };
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") api.closeMenuPopup();
    };
    const onBlur = () => {
      api.closeMenuPopup();
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("blur", onBlur);
    };
  }, []);

  const act = (fn: () => void | Promise<void>) => {
    Promise.resolve(fn()).finally(() => api.closeMenuPopup());
  };

  const openInternal = (kind: string) =>
    act(() => {
      api.newTab(`quantum://${kind}`);
    });

  const tab = snapshot?.tabs.find((t) => t.id === state?.tabId) ?? null;

  return (
    <div className="popover" role="menu" aria-label="Menü">
      {state?.menu === "main" && (
        <>
          <button className="dropdown-item" onClick={() => act(() => api.newTab())}>
            <Plus size={15} /> Yeni Sekme <span className="kbd">Ctrl T</span>
          </button>
          <button className="dropdown-item" onClick={() => act(() => api.newIncognitoTab())}>
            <EyeOff size={15} /> Gizli Sekme <span className="kbd">Ctrl ⇧ N</span>
          </button>
          <div className="dropdown-sep" />
          <button className="dropdown-item" onClick={() => openInternal("bookmarks")}>
            <Bookmark size={15} /> Yer İmleri <span className="kbd">Ctrl B</span>
          </button>
          <button className="dropdown-item" onClick={() => openInternal("history")}>
            <History size={15} /> Geçmiş <span className="kbd">Ctrl ⇧ Y</span>
          </button>
          <button className="dropdown-item" onClick={() => openInternal("downloads")}>
            <Download size={15} /> İndirmeler <span className="kbd">Ctrl J</span>
          </button>
          <div className="dropdown-sep" />
          <button className="dropdown-item" onClick={() => openInternal("settings")}>
            <Settings size={15} /> Ayarlar
          </button>
        </>
      )}

      {state?.menu === "tab" && tab && (
        <>
          <button className="dropdown-item" onClick={() => act(() => api.duplicateTab(tab.id))}>
            <Copy size={15} /> Sekmeyi Çoğalt
          </button>
          <button className="dropdown-item" onClick={() => act(() => api.togglePin(tab.id))}>
            {tab.pinned ? <PinOff size={15} /> : <Pin size={15} />}
            {tab.pinned ? "Sabitlemeyi Kaldır" : "Sekmeyi Sabitle"}
          </button>
          <div className="dropdown-sep" />
          {(snapshot?.groups ?? []).length > 0 ? (
            (snapshot?.groups ?? []).map((g) => (
              <button
                key={g.id}
                className="dropdown-item"
                onClick={() =>
                  act(() => api.assignGroup(tab.id, tab.group_id === g.id ? null : g.id))
                }
              >
                <span className="group-dot" style={{ background: g.color }} />
                {g.name}
                {tab.group_id === g.id && <span className="kbd">✓</span>}
              </button>
            ))
          ) : (
            <button
              className="dropdown-item"
              onClick={() =>
                act(async () => {
                  const gid = await api.createGroup("Genel", "#0ea5e9");
                  await api.assignGroup(tab.id, gid);
                })
              }
            >
              <FolderPlus size={15} /> Yeni Grup (Genel)
            </button>
          )}
          <div className="dropdown-sep" />
          <button className="dropdown-item" onClick={() => act(() => api.closeOthers(tab.id))}>
            <X size={15} /> Diğerlerini Kapat
          </button>
          <button
            className="dropdown-item"
            style={{ color: "var(--danger)" }}
            onClick={() => act(() => api.closeTab(tab.id))}
          >
            <Trash2 size={15} /> Sekmeyi Kapat
          </button>
        </>
      )}
    </div>
  );
}
