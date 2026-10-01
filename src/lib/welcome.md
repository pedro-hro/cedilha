# Welcome to Cedilha ç

> A quiet place to write Markdown with LaTeX. You are reading this page *inside* the app, in
> split view: the Markdown source is on the left, the rendered page on the right. Everything is
> live, so edit anything, break it, fix it.

## Contents

1. [Start here](#start)
2. [Keyboard cheat sheet](#keys)
3. [Markdown in five minutes](#markdown)
4. [Math with LaTeX](#math)
5. [Make it yours](#settings)
6. [Good to know](#notes)

---

## 1. Start here {#start}

Cedilha has three views. Switch with the tabs in the top bar or cycle with **Ctrl+P**:

| View | What you see | Good for |
| :--- | :--- | :--- |
| **edit** | only the Markdown source | writing without distraction |
| **split** | source on the left, rendered page on the right | checking math as you type |
| **preview** | only the rendered page | reading and presenting |

In **split** view the two panes stay in step: scroll either one and the other follows, block by
block, even when a tall equation sits in the middle.

**Try it now.** In split view, change the number in this equation and watch it update:

$$
E = mc^2 \qquad\Longrightarrow\qquad E \approx 9\times10^{16}\ \mathrm{J\ per\ kg}
$$

Everything you type is saved for you: once a file has a name, Cedilha writes it to disk about a
second and a half after you stop typing. A brand new document has no name yet, so press
**Ctrl+S** once to choose where it lives.

---

## 2. Keyboard cheat sheet {#keys}

Cedilha is built to be used without the mouse. These are the defaults; every one of them can be
changed in Settings.

| Action | Keys |
| :--- | :--- |
| New / open / save | `Ctrl+N` · `Ctrl+O` · `Ctrl+S` |
| Save as… | `Ctrl+Shift+S` |
| Cycle edit → split → preview | `Ctrl+P` |
| Find in the document | `Ctrl+F` |
| Next / previous match | `F3` or `Ctrl+G` · `Shift+F3` or `Ctrl+Shift+G` |
| Settings | `Ctrl+,` |
| Open this welcome page again | `F1` |
| Select the next occurrence of a word | `Ctrl+D` |
| Go to line | `Ctrl+Alt+G` |
| Minimize / maximize / close the window | `Ctrl+M` · `Ctrl+Shift+M` · `Ctrl+Q` or `Ctrl+W` |

Inside the search box, `Enter` jumps to the next match and `Shift+Enter` to the previous one.
`Esc` closes search, Settings or any dialog.

> **Tip.** In Settings, move with the arrow keys (or `j` / `k`), change a value with `←` `→`,
> press `Enter` on a shortcut and then press the new keys to rebind it.

---

## 3. Markdown in five minutes {#markdown}

### Text

Plain text, **bold**, *italic*, ***both***, ~~strikethrough~~ and `inline code`.
A backslash escapes a special character: \*not italic\*, and a price of \$5 stays a price.
End a line with a backslash\
to break it without starting a new paragraph.

### Headings

Start a line with one to six `#` characters. Add `{#some-id}` to a heading to give it a link
target, like the `{#markdown}` on this section's heading, then link to it with `[text](#some-id)`.

### Lists

- Bullets use `-`, `*` or `+`
- They can be nested
  - by indenting two spaces
  - as deep as you like
1. Numbered lists
2. count for themselves
3. so the digits you type don't matter

Task lists make good checklists:

- [x] Write the introduction
- [x] Subtract the stellar continuum
- [ ] Measure the emission lines
  - [ ] Hβ and [O III]
  - [ ] Hα and [N II]
- [ ] Make coffee ☕

### Quotes

> Quotes start with `>`.
>
> > And can be nested.

### Code

Wrap code in triple backticks and name the language:

```python
import numpy as np

wavelength = np.linspace(3500, 7500, 4000)   # angstrom
print(f"{wavelength.size} pixels, {wavelength[1] - wavelength[0]:.2f} Å each")
```

```bash
cedilha ~/notes/class.md      # open a file straight from the terminal
```

### Tables

Colons in the divider row set the alignment: left, centered, right.

| Left-aligned | Centered | Right-aligned |
| :--- | :---: | ---: |
| text | text | 1 |
| more text | more text | 22 |
| even more | even more | 333 |

### Links, footnotes and rules

A [link](https://commonmark.org/help/) and a bare address: <https://www.markdownguide.org>.
Footnotes[^fn] collect themselves at the bottom of the page.
Three dashes on their own make a horizontal rule, like the one just below.

---

## 4. Math with LaTeX {#math}

Put math between dollar signs. A single `$` pair is **inline**, like $\lambda = c/\nu$ inside a
sentence. A double `$$` pair, on its own lines, is a centered **display** equation:

$$
B_\lambda(T) = \frac{2hc^2}{\lambda^5}\,\frac{1}{e^{hc/\lambda k_B T} - 1}
$$

Several aligned lines use the `aligned` environment, with `&` marking what lines up:

$$
\begin{aligned}
\lambda_{\max} T &= b \approx 2898\ \mu\mathrm{m\,K} \\
T_\odot = 5772\ \mathrm{K} \quad&\Longrightarrow\quad \lambda_{\max} \approx 502\ \mathrm{nm}
\end{aligned}
$$

Piecewise definitions use `cases`, and matrices use `pmatrix`:

$$
\Theta(x) = \begin{cases} 0 & x < 0 \\ 1 & x \ge 0 \end{cases}
\qquad\qquad
\mathbf{R}(\theta) = \begin{pmatrix} \cos\theta & -\sin\theta \\ \sin\theta & \cos\theta \end{pmatrix}
$$

A few building blocks to get you started:

| You want | You type | You get |
| :--- | :--- | :---: |
| Fractions | `\frac{a}{b}` | $\frac{a}{b}$ |
| Powers and indices | `x^2`, `x_i`, `x_i^2` | $x^2,\ x_i,\ x_i^2$ |
| Roots | `\sqrt{x}`, `\sqrt[3]{x}` | $\sqrt{x},\ \sqrt[3]{x}$ |
| Greek letters | `\alpha \beta \Gamma \Omega` | $\alpha\ \beta\ \Gamma\ \Omega$ |
| Sums and integrals | `\sum_{i=1}^{N}`, `\int_0^\infty` | $\sum_{i=1}^{N},\ \int_0^\infty$ |
| Derivatives | `\frac{\partial f}{\partial x}` | $\frac{\partial f}{\partial x}$ |
| Vectors and hats | `\vec{v}`, `\hat{n}`, `\dot{M}` | $\vec{v},\ \hat{n},\ \dot{M}$ |
| Text inside math | `\text{if } x > 0` | $\text{if } x > 0$ |
| Units | `5\ \mathrm{km\,s^{-1}}` | $5\ \mathrm{km\,s^{-1}}$ |
| Big brackets | `\left( \frac{a}{b} \right)` | $\left( \frac{a}{b} \right)$ |

Some habits that save headaches:

- Use `\mathrm{...}` for units and names, so they are upright instead of italic.
- Use `\,` for a thin space, `\ ` for a normal one and `\quad` for a wide one.
- If a formula refuses to render, the preview shows it in red: read the message, fix the
  brace, and it renders as soon as you stop typing.
- A literal dollar sign needs a backslash (`\$`), or Cedilha will wait for a closing one.

---

## 5. Make it yours {#settings}

Press **Ctrl+,** to open Settings. Everything is driven from the keyboard, and it is saved as
you change it:

- **Theme**: Gruvbox Dark (the default), Catppuccin Mocha or Latte, Tokyo Night, or Omarchy,
  which follows the colors of your Omarchy system theme.
- **Font family, font size and text column width**: the column keeps long lines readable
  without squeezing the window.
- **Tab width, typewriter scrolling and line numbers**: typewriter scrolling keeps the line you
  are writing in the middle of the screen.
- **Open in view**: start in edit, split or preview.
- **Window buttons and rounded corners**: Cedilha has no title bar of its own. On tiling window
  managers (Hyprland, sway, i3…) these stay out of the way; on GNOME, KDE and friends they appear
  automatically, and you can force them on or off.
- **Keybindings**: every shortcut in the cheat sheet above can be rebound.

Settings live in a plain text file, `~/.config/cedilha/config.toml`. Edit it by hand with any
editor and a running Cedilha picks the change up within a second.

---

## 6. Good to know {#notes}

- **This page.** It opened once, on your first launch, as a new untitled document. Press `F1`
  whenever you want it back; it never overwrites anything of yours.
- **Opening files.** Use `Ctrl+O`, run `cedilha file.md` from a terminal, or double-click a
  Markdown file once Cedilha is installed.
- **Your work is protected.** If you try to open, create or close something with unsaved
  changes, Cedilha asks first. If another program changes the open file, autosave pauses
  instead of overwriting it, and `Ctrl+S` is how you choose to overwrite.
- **Safe by design.** Raw HTML in a document is shown as text rather than run, and links that
  could execute code are disabled, so it is safer to open Markdown files, but be careful nevertheless.
- **Images.** Pictures from `https://` addresses and embedded `data:` images render; images
  referenced by a local file path do not yet.
- **Math.** Formulas are rendered by [KaTeX](https://katex.org), which supports most of what you
  will need from LaTeX math; its [function list](https://katex.org/docs/supported.html) is the
  place to look when something is missing.

---

[^fn]: Like this one. Hover over nothing, scroll down, and here it is.



*Made with ç.*
