// Frame-driven smooth scrolling for keyboard navigation.
//
// Why this exists: WebKitGTK scrolls a key press by jumping `scrollTop` once per key-repeat
// event (~25-30 Hz), so on a 60+ Hz display motion arrives in coarse steps. Restarting a CSS
// `behavior: "smooth"` animation on every repeat is worse (each restart resets the easing).
//
// One SmoothScroller per scroll container keeps a single rAF loop that eases the real position
// toward a *destination*. Key presses only move the destination (retargeting), so the motion
// stays continuous no matter how often or how irregularly repeat events arrive. Held arrow keys
// can switch to velocity mode (`startHold`) which is independent of the repeat rate altogether.
//
// The loop only writes `scrollTop` (never reads layout between writes) and yields to the user:
// wheel, touch and pointer input cancel it.

/** Time constant (s) of the exponential approach for short moves. */
const TAU_BASE = 0.09;
/** Extra time constant per viewport of distance, so long jumps (Home/End) stay snappy. */
const TAU_PER_VIEWPORT = 0.012;
const TAU_MAX = 0.16;
/** Stop animating when closer than this many pixels to the destination. */
const EPSILON = 0.4;
/** Longest frame step we integrate (s); avoids a jump after a stall. */
const MAX_DT = 0.05;

const reducedMotion =
  typeof window !== "undefined" && typeof window.matchMedia === "function"
    ? window.matchMedia("(prefers-reduced-motion: reduce)")
    : null;

const prefersReducedMotion = () => reducedMotion?.matches ?? false;

export class SmoothScroller {
  private pos = 0;
  private dest = 0;
  private written = 0;
  private tau = TAU_BASE;
  private raf = 0;
  private last = 0;
  private holdDir = 0;
  private holdMax = 0;
  private holdAccel = 0;
  private v = 0;
  private readonly cancelOnInput = () => this.cancel();
  private readonly onScroll = () => this.adoptExternalScroll();

  constructor(readonly el: HTMLElement) {
    el.addEventListener("wheel", this.cancelOnInput, { passive: true });
    el.addEventListener("touchstart", this.cancelOnInput, { passive: true });
    el.addEventListener("pointerdown", this.cancelOnInput, { passive: true });
    el.addEventListener("scroll", this.onScroll, { passive: true });
  }

  dispose() {
    this.cancel();
    this.el.removeEventListener("wheel", this.cancelOnInput);
    this.el.removeEventListener("touchstart", this.cancelOnInput);
    this.el.removeEventListener("pointerdown", this.cancelOnInput);
    this.el.removeEventListener("scroll", this.onScroll);
  }

  get active(): boolean {
    return this.raf !== 0;
  }

  /** Where the container is heading (its current position when idle). */
  destination(): number {
    return this.raf ? this.dest : this.el.scrollTop;
  }

  private max(): number {
    return Math.max(0, this.el.scrollHeight - this.el.clientHeight);
  }

  private clamp(y: number): number {
    return Math.min(this.max(), Math.max(0, y));
  }

  /** Ease to an absolute scrollTop. Calling this repeatedly just retargets the motion. */
  scrollTo(y: number, instant = false) {
    y = this.clamp(y);
    if (instant || prefersReducedMotion()) {
      this.cancel();
      this.el.scrollTop = y;
      return;
    }
    if (!this.raf) this.pos = this.el.scrollTop;
    this.dest = y;
    const h = Math.max(1, this.el.clientHeight);
    this.tau = Math.min(TAU_MAX, TAU_BASE + TAU_PER_VIEWPORT * Math.min(6, Math.abs(y - this.pos) / h));
    this.holdDir = 0;
    this.kick();
  }

  scrollBy(dy: number) {
    this.scrollTo(this.destination() + dy);
  }

  /**
   * Continuous scrolling at up to `maxSpeed` px/s in direction `dir` (+1/-1), accelerating
   * over ~0.25 s. Returns false when motion is reduced (caller should step instead).
   */
  startHold(dir: 1 | -1, maxSpeed: number): boolean {
    if (prefersReducedMotion()) return false;
    if (this.holdDir === dir) return true;
    if (!this.raf) {
      this.pos = this.el.scrollTop;
      this.v = 0;
    } else if (Math.sign(this.v) !== dir) {
      this.v = 0;
    }
    this.holdDir = dir;
    this.holdMax = maxSpeed;
    this.holdAccel = maxSpeed / 0.25;
    this.v = dir * Math.max(Math.abs(this.v), maxSpeed * 0.35);
    this.dest = this.pos;
    this.kick();
    return true;
  }

  /** Leave velocity mode, gliding to a stop instead of halting abruptly. */
  endHold() {
    if (!this.holdDir) return;
    this.holdDir = 0;
    this.tau = TAU_BASE;
    this.dest = this.clamp(this.pos + this.v * this.tau);
    this.v = 0;
    this.kick();
  }

  cancel() {
    if (this.raf) cancelAnimationFrame(this.raf);
    this.raf = 0;
    this.holdDir = 0;
    this.v = 0;
    this.last = 0;
  }

  private kick() {
    if (!this.raf) {
      this.last = 0;
      this.raf = requestAnimationFrame(this.tick);
    }
  }

  // Something else moved the container mid-flight (CodeMirror height-estimate corrections,
  // scroll sync): follow it instead of fighting it or aborting the motion.
  private adoptExternalScroll() {
    if (!this.raf) return;
    const d = this.el.scrollTop - this.written;
    if (Math.abs(d) > 1) {
      this.pos += d;
      this.dest += d;
      this.written = this.el.scrollTop;
    }
  }

  private readonly tick = (now: number) => {
    const dt = this.last ? Math.min(MAX_DT, Math.max(0.001, (now - this.last) / 1000)) : 1 / 60;
    this.last = now;

    let done = false;
    if (this.holdDir) {
      const speed = Math.min(this.holdMax, Math.abs(this.v) + this.holdAccel * dt);
      this.v = this.holdDir * speed;
      this.pos += this.v * dt;
      const clamped = this.clamp(this.pos);
      if (clamped !== this.pos) {
        this.pos = clamped;
        this.v = 0;
        this.holdDir = 0;
        this.dest = clamped;
        done = true;
      } else {
        this.dest = this.pos;
      }
    } else {
      this.dest = this.clamp(this.dest);
      const diff = this.dest - this.pos;
      if (Math.abs(diff) < EPSILON) {
        this.pos = this.dest;
        done = true;
      } else {
        this.pos += diff * (1 - Math.exp(-dt / this.tau));
      }
    }

    this.el.scrollTop = this.pos;
    this.written = this.el.scrollTop;
    // The browser clamped or rounded us somewhere unexpected: resynchronise.
    if (Math.abs(this.written - this.pos) > 1.5) this.pos = this.written;

    if (done && !this.holdDir) {
      this.raf = 0;
      this.last = 0;
      this.v = 0;
    } else {
      this.raf = requestAnimationFrame(this.tick);
    }
  };
}

const scrollers = new WeakMap<HTMLElement, SmoothScroller>();

/** The shared SmoothScroller of an element (created on first use). */
export function smoothScrollerFor(el: HTMLElement): SmoothScroller {
  let s = scrollers.get(el);
  if (!s) {
    s = new SmoothScroller(el);
    scrollers.set(el, s);
  }
  return s;
}

/** Smoothly scroll `child` into view inside `container` with a small margin ("nearest"). */
export function smoothEnsureVisible(container: HTMLElement, child: HTMLElement, margin = 8) {
  const s = smoothScrollerFor(container);
  const c = container.getBoundingClientRect();
  const r = child.getBoundingClientRect();
  const base = s.destination();
  const shift = container.scrollTop - base; // rects are relative to the real position
  const top = r.top + shift - c.top;
  const bottom = r.bottom + shift - c.top;
  if (top < margin) s.scrollTo(base + top - margin);
  else if (bottom > container.clientHeight - margin) s.scrollTo(base + bottom - container.clientHeight + margin);
}
