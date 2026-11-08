# Cedilha (`ç`)

A minimalist, keyboard-first **Markdown & LaTeX** editor built with **Rust**, **Tauri v2** and **Svelte**, made for Arch Linux with Omarchy / Hyprland.

*Inspired by the simplicity of GNOME's Apostrophe.*

---

## Features

- **Live LaTeX math:** inline `$E = mc^2$` and block `$$\int f(x) dx$$` formulas rendered with KaTeX in the preview.
- **Omarchy / Hyprland native:** borderless window with a draggable header, so your window manager handles borders and gaps. The `omarchy` theme follows the active Omarchy system theme live.
- **Typewriter scrolling:** optionally keeps the cursor line vertically centered (toggle in Settings).
- **Smooth keyboard scrolling** in the editor and the preview.
- **Live stats:** word, character and line counts and estimated reading time.
- **View modes:**
  - **Edit:** full-width Markdown editor.
  - **Split:** source on the left, rendered preview on the right, scrolled in step.
  - **Preview:** full-width rendered page.
- **Find:** in-document search with next/previous match.
- **Themes:** `omarchy` (follows the system), Catppuccin Mocha, Catppuccin Latte, Tokyo Night and Gruvbox Dark (default).
- **Auto-save:** once a file has a name, it is written to disk shortly after you stop typing.
- **Configurable:** preferences and shortcut overrides live in a `config.toml` and in the in-app Settings (`Ctrl+,`).

---

## Installation

### Arch Linux (AUR)

Once published on the AUR:

```bash
yay -S cedilha        # latest release
yay -S cedilha-git    # latest git checkout
```

### Local Arch package

```bash
git clone https://github.com/pedro-hro/cedilha.git
cd cedilha/packaging/arch && makepkg -si
```

### Build from source

Requires [Bun](https://bun.sh), a Rust toolchain, and the Tauri 2 Linux dependencies (`webkit2gtk-4.1`, `gtk3`).

```bash
git clone https://github.com/pedro-hro/cedilha.git
cd cedilha
bun install && bun run build && cd src-tauri && cargo build --release --features custom-protocol
```

The binary is written to `src-tauri/target/release/cedilha`.

### Development

```bash
bun run tauri dev
```

---

## Keyboard Shortcuts

Defaults (all rebindable in Settings):

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + P` | Cycle view (Edit → Split → Preview) |
| `Ctrl + N` | New document |
| `Ctrl + O` | Open document |
| `Ctrl + S` | Save |
| `Ctrl + Shift + S` | Save as |
| `Ctrl + F` | Find |
| `F3` / `Ctrl + G` | Find next |
| `Shift + F3` / `Ctrl + Shift + G` | Find previous |
| `Ctrl + ,` | Settings |
| `F1` | Welcome page |
| `Ctrl + M` / `Ctrl + Shift + M` | Minimize / maximize window |
| `Ctrl + Q` / `Ctrl + W` | Close window |

---

## License

[MIT](LICENSE) © 2026 Pedro Henrique Reis de Oliveira
