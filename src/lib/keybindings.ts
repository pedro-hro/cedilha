// Customizable keyboard shortcuts.
//
// Defaults live here; the user's config.toml only stores overrides (`[keybindings]`).
// A combo is written like "Ctrl+Shift+S", "Alt+Enter" or "F3". "Ctrl" also matches the
// Meta key so the same bindings work on any platform.

export type ActionId =
  | "cycle_view"
  | "new"
  | "open"
  | "save"
  | "save_as"
  | "settings"
  | "find"
  | "find_next"
  | "find_prev";

export interface ActionDef {
  id: ActionId;
  label: string;
  defaults: string[];
}

export const ACTIONS: ActionDef[] = [
  { id: "cycle_view", label: "Cycle view (Edit / Split / Preview)", defaults: ["Ctrl+P"] },
  { id: "new", label: "New document", defaults: ["Ctrl+N"] },
  { id: "open", label: "Open document", defaults: ["Ctrl+O"] },
  { id: "save", label: "Save", defaults: ["Ctrl+S"] },
  { id: "save_as", label: "Save as…", defaults: ["Ctrl+Shift+S"] },
  { id: "settings", label: "Settings", defaults: ["Ctrl+,"] },
  { id: "find", label: "Find", defaults: ["Ctrl+F"] },
  { id: "find_next", label: "Find next", defaults: ["F3", "Ctrl+G"] },
  { id: "find_prev", label: "Find previous", defaults: ["Shift+F3", "Ctrl+Shift+G"] },
];

export type Bindings = Record<ActionId, string[]>;

export interface Combo {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  key: string; // lowercase, as in KeyboardEvent.key
}

export function parseCombo(text: string): Combo | null {
  const m = /^((?:(?:ctrl|control|alt|shift|mod|cmd|meta)\+)*)(.+)$/i.exec(text.trim());
  if (!m) return null;
  const mods = m[1].toLowerCase();
  const key = normalizeKey(m[2]);
  if (!key) return null;
  return {
    ctrl: /ctrl|control|mod|cmd|meta/.test(mods),
    alt: mods.includes("alt"),
    shift: mods.includes("shift"),
    key,
  };
}

function normalizeKey(key: string): string {
  if (key === " ") return "space";
  return key.trim().toLowerCase();
}

export function matches(e: KeyboardEvent, combo: Combo): boolean {
  return (
    (e.ctrlKey || e.metaKey) === combo.ctrl &&
    e.altKey === combo.alt &&
    e.shiftKey === combo.shift &&
    normalizeKey(e.key) === combo.key
  );
}

/** Canonical text for a key press, or null for a bare modifier / an unusable combo. */
export function comboFromEvent(e: KeyboardEvent): string | null {
  if (["Control", "Shift", "Alt", "Meta", "AltGraph", "Dead"].includes(e.key)) return null;
  const ctrl = e.ctrlKey || e.metaKey;
  // A plain letter can't be a shortcut (it would break typing); function keys can.
  if (!ctrl && !e.altKey && !/^F\d{1,2}$/.test(e.key)) return null;
  const parts: string[] = [];
  if (ctrl) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  const key = e.key === " " ? "Space" : e.key.length === 1 ? e.key.toUpperCase() : e.key;
  parts.push(key);
  return parts.join("+");
}

/** Normalized form used to detect two actions sharing a shortcut. */
export function comboId(text: string): string | null {
  const c = parseCombo(text);
  return c ? `${c.ctrl ? "c" : ""}${c.alt ? "a" : ""}${c.shift ? "s" : ""}:${c.key}` : null;
}

/** Display parts for a combo, e.g. "Ctrl+Shift+S" -> ["Ctrl", "Shift", "S"]. */
export function comboParts(text: string): string[] {
  const m = /^((?:(?:ctrl|control|alt|shift|mod|cmd|meta)\+)*)(.+)$/i.exec(text.trim());
  if (!m) return [text];
  const mods = m[1].split("+").filter(Boolean);
  const key = m[2].length === 1 ? m[2].toUpperCase() : m[2];
  return [...mods, key];
}

// CodeMirror key names are case-sensitive for named keys ("F3", "Enter", "ArrowUp").
function codeMirrorKeyName(text: string, lower: string): string {
  if (lower === "space") return "Space";
  if (lower.length === 1) return lower;
  if (/^f\d{1,2}$/.test(lower)) return lower.toUpperCase();
  const raw = text.trim().slice(text.trim().length - lower.length);
  return raw[0].toUpperCase() + raw.slice(1);
}

/** CodeMirror key name for a combo ("Ctrl-Shift-g"), or null if it can't be parsed. */
export function toCodeMirror(text: string): string | null {
  const c = parseCombo(text);
  if (!c) return null;
  const parts: string[] = [];
  if (c.ctrl) parts.push("Ctrl");
  if (c.alt) parts.push("Alt");
  if (c.shift) parts.push("Shift");
  parts.push(codeMirrorKeyName(text, c.key));
  return parts.join("-");
}

/** Defaults overlaid with the user's overrides; unparseable overrides are ignored. */
export function resolveBindings(overrides?: Record<string, string[]>): Bindings {
  const out = {} as Bindings;
  for (const a of ACTIONS) {
    const custom = overrides?.[a.id];
    const valid = Array.isArray(custom) ? custom.filter((c) => parseCombo(c)) : [];
    out[a.id] = valid.length > 0 ? valid : [...a.defaults];
  }
  return out;
}
