# Cedilha (`ç`)

An opinionated, distraction-free **Markdown & LaTeX** editor built with **Rust** and **Tauri v2**, designed with the **Omarchy aesthetic** for Arch Linux and Hyprland.

*Inspired by the simplicity of GNOME's Apostrophe, re-engineered in Rust and web technologies for rock-solid stability and instant math rendering.*

---

## Features

- **Live LaTeX Math Rendering:** Type inline `$E = mc^2$` or block `$$\int f(x) dx$$` formulas and watch them render instantly with KaTeX. Click any formula to edit the raw LaTeX.
- **Omarchy / Hyprland Native:** Borderless window chrome with draggable header, letting your window manager handle borders and gaps.
- **Distraction-Free Typewriter Scrolling:** Keeps the active cursor line vertically centered on the viewport.
- **Real-Time Word & Reading Stats:** Live word count, character count, line count, and estimated reading time.
- **Apostrophe-Style View Modes:**
  - **Edit:** Distraction-free, full-width Markdown editor.
  - **Split:** Side-by-side view with live rendered HTML and KaTeX preview on the right.
  - **Preview:** Full-width rendered reading view.
- **Theming:** Built-in community palettes:
  - Catppuccin Mocha (Dark)
  - Catppuccin Latte (Light)
  - Tokyo Night
  - Gruvbox Dark
- **Auto-Save & Safe File I/O:** Debounced background autosave with dirty-state tracking.

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + 1` | Edit Mode (Code only) |
| `Ctrl + 2` | Split Mode (Side-by-side) |
| `Ctrl + 3` | Preview Mode (Rendered view) |
| `Ctrl + P` | Cycle View Modes (Edit ↔ Split ↔ Preview) |
| `Ctrl + N` | New Document |
| `Ctrl + O` | Open Document |
| `Ctrl + S` | Save Document |
| `Ctrl + Shift + S` | Save As |
| `Ctrl + ,` | Open Settings |

---

## Getting Started

### Development
```bash
# Run both the Svelte frontend and the Tauri Rust backend
bun run tauri dev
```

### Build Production Binary
```bash
bun run tauri build
```
The compiled binary will be placed at `src-tauri/target/release/cedilha`.
