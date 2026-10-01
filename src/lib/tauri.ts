// Typed wrappers around the Rust IPC commands defined in src-tauri.
// Centralizing the `invoke` calls keeps the rest of the app free of magic
// command-name strings and keeps type-checking on our side of the bridge.

import { invoke } from "@tauri-apps/api/core";

export interface Prefs {
  theme: string;
  font_size: number;
  tab_size: number;
  wrap: boolean;
  typewriter: boolean;
  line_numbers?: boolean;
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

export function getPrefs(): Promise<Prefs> {
  return invoke("get_prefs");
}

export function savePrefs(prefs: Prefs): Promise<void> {
  return invoke("save_prefs", { newPrefs: prefs });
}
