// Builds a fully-configured CodeMirror 6 editor state for Cedilha.
//
// Composition: Markdown language support + permanent word wrap + typewriter
// scrolling + keyboard navigation, with reactive Compartments for runtime settings.

import { EditorState, type Extension } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { standardKeymap, indentWithTab } from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { search } from "@codemirror/search";
import { highlightActiveLine } from "@codemirror/view";
import {
  type EditorOptions,
  tabSizeCompartment,
  typewriterCompartment,
  lineNumbersCompartment,
  searchKeysCompartment,
  getSearchKeysExtension,
  getTabSizeExtension,
  getLineNumbersExtension,
} from "./codemirror/options";
import { createTypewriterExtension } from "./codemirror/typewriter";

export interface FullEditorOptions extends EditorOptions {
  content: string;
  updateListener?: Extension;
}

// Clean base theme that inherits variables from :root[data-theme] in app.css
const baseTheme = EditorView.theme({
  "&": {
    height: "100%",
    backgroundColor: "transparent",
    color: "var(--fg)",
  },
  ".cm-scroller": {
    fontFamily: "var(--font-mono)",
    lineHeight: "1.6",
  },
  ".cm-content": {
    caretColor: "var(--accent)",
    fontFamily: "var(--font-mono)",
    lineHeight: "1.6",
  },
  // Lines span the full editor width so the active-line band is edge to edge. The right
  // padding keeps text inside the configured reading column (--editor-max-width).
  ".cm-line": {
    lineHeight: "1.6",
    padding: "0 max(32px, calc(100% - var(--editor-max-width, 1040px))) 0 24px",
  },
  ".cm-gutters ~ .cm-content .cm-line": {
    paddingLeft: "16px",
  },
  "&.cm-focused .cm-cursor": {
    borderLeftColor: "var(--accent)",
    borderLeftWidth: "2px",
  },
  ".cm-selectionBackground, ::selection": {
    backgroundColor: "var(--selection) !important",
  },
  ".cm-gutters": {
    backgroundColor: "transparent",
    color: "var(--fg-dim)",
    border: "none",
    marginRight: "0",
    paddingLeft: "0",
    userSelect: "none",
    minHeight: "0 !important",
    height: "fit-content !important",
    alignSelf: "flex-start",
  },
  ".cm-lineNumbers": {
    paddingRight: "0",
    minHeight: "0 !important",
    height: "fit-content !important",
  },
  ".cm-lineNumbers .cm-gutterElement": {
    fontFamily: "var(--font-mono)",
    lineHeight: "1.65",
    padding: "0 8px 0 16px",
    minWidth: "28px",
    textAlign: "right",
    color: "var(--fg-dim)",
  },
  ".cm-activeLine": {
    backgroundColor: "var(--active-line, rgba(255, 255, 255, 0.035))",
  },
  ".cm-panels": {
    backgroundColor: "var(--bg-elev)",
    color: "var(--fg)",
    borderBottom: "1px solid var(--border)",
  },
  ".cm-search": {
    display: "flex",
    flexWrap: "wrap",
    alignItems: "center",
    gap: "6px",
    padding: "8px 12px !important",
    fontFamily: "var(--font-mono)",
    fontSize: "13px",
  },
  ".cm-search br": { display: "none" },
  // Regular-expression search isn't useful for prose/LaTeX; keep the panel minimal.
  ".cm-search label:has(input[name=re])": { display: "none" },
  ".cm-search input.cm-textfield": {
    backgroundColor: "var(--bg)",
    color: "var(--fg)",
    border: "1px solid var(--border)",
    borderRadius: "var(--radius)",
    padding: "3px 8px",
    margin: "0",
    outline: "none",
  },
  ".cm-search input.cm-textfield:focus": { borderColor: "var(--accent)" },
  ".cm-search button.cm-button": {
    backgroundColor: "var(--bg-surface)",
    backgroundImage: "none",
    color: "var(--fg)",
    border: "1px solid var(--border)",
    borderRadius: "var(--radius)",
    padding: "3px 10px",
    margin: "0",
  },
  ".cm-search button.cm-button:hover": { borderColor: "var(--accent)" },
  ".cm-search label": { color: "var(--fg-dim)", display: "inline-flex", alignItems: "center", gap: "4px" },
  ".cm-search [name=close]": { color: "var(--fg-dim)", background: "none", border: "none", cursor: "pointer" },
  ".cm-searchMatch": {
    backgroundColor: "var(--selection)",
    outline: "1px solid var(--accent)",
  },
  ".cm-searchMatch.cm-searchMatch-selected": {
    backgroundColor: "var(--accent)",
    color: "var(--bg)",
  },
  ".cm-activeLineGutter": {
    backgroundColor: "var(--active-line, rgba(255, 255, 255, 0.035))",
    color: "var(--accent)",
    fontWeight: "600",
  },
});

export function createEditor(opts: FullEditorOptions): EditorState {
  const extensions: Extension[] = [
    baseTheme,
    highlightActiveLine(),
    markdown({ base: markdownLanguage }),
    // Fixed, opinionated word-wrapping
    EditorView.lineWrapping,
    tabSizeCompartment.of(getTabSizeExtension(opts.tabSize)),
    typewriterCompartment.of(createTypewriterExtension(opts.typewriter)),
    lineNumbersCompartment.of(getLineNumbersExtension(opts.lineNumbers ?? false)),
    search({ top: true }),
    searchKeysCompartment.of(getSearchKeysExtension(opts.findNextKeys, opts.findPrevKeys)),
    keymap.of([...standardKeymap, indentWithTab]),
    EditorState.allowMultipleSelections.of(true),
  ];

  if (opts.updateListener) {
    extensions.push(opts.updateListener);
  }

  return EditorState.create({
    doc: opts.content,
    extensions,
  });
}
