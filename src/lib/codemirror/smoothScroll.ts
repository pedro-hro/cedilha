// Smooth cursor-follow for CodeMirror.
//
// CodeMirror brings the cursor into view by assigning `scrollTop` immediately, once per key
// press (so once per auto-repeat event while an arrow key is held). This installs a
// `scrollHandler` that computes the same target but hands it to the shared SmoothScroller, so
// repeated cursor moves retarget one continuous animation. It covers every path that goes through
// `EditorView.scrollIntoView`: arrows, PageUp/PageDown, Home/End, typing, search results and the
// typewriter extension (`y: "center"`).

import type { Extension } from "@codemirror/state";
import { EditorView, ViewPlugin } from "@codemirror/view";
import { SmoothScroller } from "../smoothScroll";

// Keep the cursor at least this many line-heights from the edge when following it ("nearest").
const COMFORT_LINES = 1.5;

const plugin = ViewPlugin.define((view) => {
  const scroller = new SmoothScroller(view.scrollDOM);
  return { scroller, destroy: () => scroller.dispose() };
});

export function smoothScrollExtension(): Extension {
  return [
    plugin,
    EditorView.scrollHandler.of((view, range, o) => {
      const scroller = view.plugin(plugin)?.scroller;
      const dom = view.scrollDOM;
      if (!scroller || dom.scrollHeight <= dom.clientHeight) return false;

      const side = range.assoc || (range.head > range.anchor ? -1 : 1);
      let rect = view.coordsAtPos(range.head, side);
      if (!rect) return false; // far outside the rendered viewport: let CodeMirror jump
      if (!range.empty) {
        const other = view.coordsAtPos(range.anchor, range.anchor > range.head ? -1 : 1);
        if (other) {
          rect = {
            left: rect.left,
            right: rect.right,
            top: Math.min(rect.top, other.top),
            bottom: Math.max(rect.bottom, other.bottom),
          };
        }
      }

      // `rect` is relative to where the scroller is *now*; express it for where it is heading.
      const base = scroller.destination();
      const shift = dom.scrollTop - base;
      const bounds = dom.getBoundingClientRect();
      const height = dom.clientHeight;
      const top = rect.top + shift - bounds.top;
      const bottom = rect.bottom + shift - bounds.top;
      const margin = Math.max(-height, Math.min(height, o.yMargin));
      const head = range.head < range.anchor ? -1 : 1;

      let move = 0;
      if (o.y === "nearest") {
        const m = Math.max(margin, Math.min(view.defaultLineHeight * COMFORT_LINES, height / 4));
        if (top < m) {
          move = top - m;
          if (head > 0 && bottom > height + move) move = bottom - height + m;
        } else if (bottom > height - m) {
          move = bottom - height + m;
          if (head < 0 && top - move < 0) move = top - m;
        }
      } else {
        const rectHeight = bottom - top;
        const targetTop =
          o.y === "center" && rectHeight <= height
            ? top + rectHeight / 2 - height / 2
            : o.y === "start" || (o.y === "center" && head < 0)
              ? top - margin
              : bottom - height + margin;
        move = targetTop;
      }

      if (move) scroller.scrollTo(base + move);
      return true;
    }),
  ];
}
