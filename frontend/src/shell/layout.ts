import { loadPrefs } from '../ui/prefs';

export type Layout = '2d' | 'split' | '3d';
const KEY = 'formfabric.layout';
const isLayout = (v: unknown): v is Layout => v === '2d' || v === 'split' || v === '3d';

export class LayoutState {
  private value: Layout;
  private readonly listeners = new Set<(l: Layout) => void>();

  constructor() {
    let saved: unknown = null;
    try { saved = localStorage.getItem(KEY); } catch { saved = null; }
    const prefs = loadPrefs();
    this.value = prefs.rememberLayout && isLayout(saved) ? saved : prefs.defaultLayout;
  }

  get(): Layout { return this.value; }

  set(next: Layout): void {
    if (next === this.value) return;
    this.value = next;
    try { localStorage.setItem(KEY, next); } catch { /* this session only */ }
    for (const fn of this.listeners) fn(next);
    document.dispatchEvent(new CustomEvent('ff:layout', { detail: { layout: next } }));
  }

  onChange(fn: (l: Layout) => void): () => void {
    this.listeners.add(fn);
    return () => { this.listeners.delete(fn); };
  }
}
