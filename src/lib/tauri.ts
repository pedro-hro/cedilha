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
}

export interface LoadResult {
  path: string | null;
  content: string;
  prefs: Prefs;
}

export function openFile(): Promise<LoadResult> {
  return invoke("open_file");
}

export function readDocument(path: string): Promise<string> {
  return invoke("read_document", { path });
}

export function saveDocument(path: string, content: string): Promise<void> {
  return invoke("save_document", { path, content });
}

export function saveAsFile(content: string): Promise<string | null> {
  return invoke("save_as_file", { content });
}

export function newDocument(): Promise<string> {
  return invoke("new_document");
}

export function toggleMinimize(): Promise<void> {
  return invoke("toggle_minimize");
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
