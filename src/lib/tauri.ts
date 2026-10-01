// Typed wrappers around the Rust IPC commands defined in src-tauri.
// Centralizing the `invoke` calls keeps the rest of the app free of magic
// command-name strings and keeps type-checking on our side of the bridge.

import { invoke } from "@tauri-apps/api/core";

export interface Prefs {
  theme: string;
  font_size: number;
  tab_size: number;
  typewriter: boolean;
  line_numbers?: boolean;
  font_family?: string;
  editor_max_width?: number;
  /** View on launch: "edit" | "split" | "preview". */
  default_view?: string;
  /** Top-bar window buttons: "auto" | "always" | "never". */
  window_controls?: string;
  /** App-drawn rounded corners: "auto" | "always" | "never". */
  rounded_corners?: string;
  /** The welcome page opens once, on first launch. */
  welcome_seen?: boolean;
  /** Keybinding overrides: action id -> key combos. See lib/keybindings.ts. */
  keybindings?: Record<string, string[]>;
}

export interface LoadResult {
  path: string | null;
  content: string;
  prefs: Prefs;
}

export function openFile(): Promise<LoadResult> {
  return invoke("open_file");
}

/** The file named on the command line, if any (consumed once). */
export function initialFile(): Promise<LoadResult | null> {
  return invoke("initial_file");
}

export function readDocument(path: string): Promise<string> {
  return invoke("read_document", { path });
}

/** `force` overwrites even if another program changed the file since we last touched it. */
export function saveDocument(path: string, content: string, force = false): Promise<void> {
  return invoke("save_document", { path, content, force });
}

export type UnsavedChoice = "save" | "discard" | "cancel";

export function exitApp(): Promise<void> {
  return invoke("exit_app");
}

export function cancelClose(): Promise<void> {
  return invoke("cancel_close");
}

export function saveAsFile(content: string): Promise<string | null> {
  return invoke("save_as_file", { content });
}

export function newDocument(): Promise<string> {
  return invoke("new_document");
}

export function minimizeWindow(): Promise<void> {
  return invoke("minimize_window");
}

/** Open an http(s)/mailto link in the system browser or mail client. */
export function openUrl(url: string): Promise<void> {
  return invoke("open_url", { url });
}

export function toggleMaximize(): Promise<void> {
  return invoke("toggle_maximize");
}

export function isTilingWm(): Promise<boolean> {
  return invoke("is_tiling_wm");
}

export interface OmarchyTheme {
  id: string;
  name: string;
  display_name: string;
  is_light: boolean;
  bg: string;
  bg_elev: string;
  bg_surface: string;
  fg: string;
  fg_dim: string;
  accent: string;
  accent2: string;
  danger: string;
  success: string;
  warning: string;
  border: string;
  selection: string;
  marker: string;
  active_line: string;
}

export function getPrefs(): Promise<Prefs> {
  return invoke("get_prefs");
}

export function savePrefs(prefs: Prefs): Promise<void> {
  return invoke("save_prefs", { newPrefs: prefs });
}

export function getOmarchyTheme(): Promise<OmarchyTheme | null> {
  return invoke("get_omarchy_theme");
}

export function renderMarkdown(content: string): Promise<string> {
  return invoke("render_markdown", { content });
}
