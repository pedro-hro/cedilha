// KaTeX-backed live math rendering for CodeMirror 6.
//
// Recognizes `$…$` inline math and `$$…$$` block math and renders it with
// KaTeX. Single-line math smoothly collapses into an inline rendered formula
// when the cursor leaves, and expands when clicked or navigated to.
// Multi-line block math renders a clean KaTeX widget immediately below the formula.

import type { Extension } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
  WidgetType,
} from "@codemirror/view";
import katex from "katex";

class MathWidget extends WidgetType {
  constructor(
    readonly latex: string,
    readonly isBlock: boolean,
  ) {
    super();
  }

  toDOM(view: EditorView): HTMLElement {
    const wrap = document.createElement(this.isBlock ? "div" : "span");
    wrap.className = this.isBlock ? "cm-math-block" : "cm-math-inline";
    wrap.setAttribute("title", this.latex);
    wrap.style.cursor = "pointer";

    try {
      wrap.innerHTML = katex.renderToString(this.latex, {
        throwOnError: false,
        displayMode: this.isBlock,
      });
    } catch {
      wrap.textContent = this.latex;
      wrap.classList.add("cm-math-error");
    }

    // Clicking the rendered formula positions the cursor inside to edit raw LaTeX.
    wrap.addEventListener("mousedown", (e) => {
      e.preventDefault();
      try {
        const pos = view.posAtDOM(wrap);
        view.dispatch({
          selection: { anchor: Math.min(pos, view.state.doc.length) },
        });
        view.focus();
      } catch {
        // Fallback
      }
    });

    return wrap;
  }

  override eq(other: MathWidget): boolean {
    return other.latex === this.latex && other.isBlock === this.isBlock;
  }

  override ignoreEvent(): boolean {
    return false;
  }
}

interface MathSpan {
  from: number;
  to: number;
  latex: string;
  isBlock: boolean;
}

function findMathSpans(text: string): MathSpan[] {
  const spans: MathSpan[] = [];

  // 1. Block math: $$...$$ (can span across multiple lines)
  const blockRegex = /\$\$([\s\S]+?)\$\$/g;
  let match: RegExpExecArray | null;
  while ((match = blockRegex.exec(text)) !== null) {
    const latex = match[1].trim();
    if (latex.length > 0) {
      spans.push({
        from: match.index,
        to: match.index + match[0].length,
        latex,
        isBlock: true,
      });
    }
  }

  // 2. Inline math: $...$ (single line, non-empty, not escaped)
  const inlineRegex = /(?<!\\)\$(?!\$)([^\$\n]+?)(?<!\\)\$/g;
  while ((match = inlineRegex.exec(text)) !== null) {
    const from = match.index;
    const to = match.index + match[0].length;
    const latex = match[1].trim();

    // Skip if inside an existing block span
    const insideBlock = spans.some((s) => s.isBlock && from >= s.from && to <= s.to);
    if (!insideBlock && latex.length > 0) {
      spans.push({
        from,
        to,
        latex,
        isBlock: false,
      });
    }
  }

  // Sort by starting document offset
  spans.sort((a, b) => a.from - b.from);
  return spans;
}

function buildDecorations(view: EditorView): DecorationSet {
  const doc = view.state.doc;
  const docText = doc.toString();
  const spans = findMathSpans(docText);
  const selection = view.state.selection.main;
  const selFrom = selection.from;
  const selTo = selection.to;

  const decorations = [];

  for (const span of spans) {
    // If cursor touches or is inside the math span, keep it expanded for editing
    const isCursorInside = selTo >= span.from && selFrom <= span.to;
    if (isCursorInside) {
      continue;
    }

    const startLine = doc.lineAt(span.from).number;
    const endLine = doc.lineAt(span.to).number;

    if (startLine === endLine) {
      // Single-line math (both $...$ and single-line $$...$$):
      // Safe to replace since it does NOT cross line boundaries
      decorations.push(
        Decoration.replace({
          widget: new MathWidget(span.latex, span.isBlock),
        }).range(span.from, span.to),
      );
    } else {
      // Multi-line block math: place a preview widget directly at the end of the block
      decorations.push(
        Decoration.widget({
          widget: new MathWidget(span.latex, true),
          side: 1,
        }).range(span.to),
      );
    }
  }

  decorations.sort((a, b) => a.from - b.from);
  return Decoration.set(decorations, true);
}

export const mathPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = buildDecorations(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.selectionSet || update.viewportChanged) {
        this.decorations = buildDecorations(update.view);
      }
    }
  },
  {
    decorations: (v) => v.decorations,
  },
);

export function mathExtension(): Extension {
  return mathPlugin;
}
