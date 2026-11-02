import { loadPrefs, updatePrefs, type SavedLayout } from '../ui/prefs';
import type { LayoutState } from './layout';

const RATIO_KEY = 'formfabric.layout.split';
const NAME_KEY = 'formfabric.layout.name';

export const DRAWERS: ReadonlyArray<readonly [string, string]> = [
  ['library', '.ff-lib-toggle'],
  ['fabric', '.ff-fb-toggle'],
  ['body', '.ff-bd-toggle'],
  ['trims', '.ff-tr-toggle'],
  ['output', '.ou-toggle-btn'],
  ['timeline', '.ff-an-toggle'],
];

const closed = { open: false, h: 280 };

export const DEFAULT_LAYOUTS: Readonly<Record<string, SavedLayout>> = {
  Pattern: { layout: '2d', ratio: 0.5, drawers: [], dock: closed },
  'Pattern and 3D': { layout: 'split', ratio: 0.5, drawers: [], dock: closed },
  Drape: { layout: '3d', ratio: 0.5, drawers: ['body'], dock: closed },
  Animate: { layout: 'split', ratio: 0.6, drawers: ['timeline'], dock: closed },
  Output: { layout: '3d', ratio: 0.5, drawers: ['output'], dock: closed },
  Library: { layout: 'split', ratio: 0.5, drawers: ['library'], dock: closed },
};

const toggleOf = (selector: string): HTMLButtonElement | null => document.querySelector<HTMLButtonElement>(selector);
const expanded = (selector: string): boolean => toggleOf(selector)?.getAttribute('aria-expanded') === 'true';

export class WorkspaceLayouts {
  private readonly listeners = new Set<() => void>();

  constructor(private readonly layout: LayoutState) {}

  onChange(fn: () => void): void { this.listeners.add(fn); }

  names(): string[] {
    const mine = loadPrefs().layouts;
    return [...Object.keys(DEFAULT_LAYOUTS).filter((n) => !(n in mine)), ...Object.keys(mine)];
  }

  isDefault(name: string): boolean { return name in DEFAULT_LAYOUTS && !(name in loadPrefs().layouts); }

  get(name: string): SavedLayout | undefined { return loadPrefs().layouts[name] ?? DEFAULT_LAYOUTS[name]; }

  active(): string | null {
    try { return localStorage.getItem(NAME_KEY); } catch { return null; }
  }

  capture(): SavedLayout {
    const html = document.documentElement;
    const ratio = Number(localStorage.getItem(RATIO_KEY));
    const h = Number(html.getAttribute('data-ft-dock-h'));
    return {
      layout: this.layout.get(),
      ratio: Number.isFinite(ratio) && ratio >= 0.15 && ratio <= 0.85 ? ratio : 0.5,
      drawers: DRAWERS.filter(([, sel]) => expanded(sel)).map(([id]) => id),
      dock: { open: html.getAttribute('data-ft-dock') === 'open', h: Number.isFinite(h) && h >= 140 ? h : 280 },
    };
  }

  save(name: string): void {
    updatePrefs({ layouts: { ...loadPrefs().layouts, [name]: this.capture() } });
    this.remember(name);
  }

  remove(name: string): boolean {
    const mine = { ...loadPrefs().layouts };
    if (!(name in mine)) return false;
    delete mine[name];
    updatePrefs({ layouts: mine });
    if (this.active() === name) this.remember(null);
    else this.emit();
    return true;
  }

  apply(name: string): boolean {
    const s = this.get(name);
    if (!s) return false;
    this.layout.set(s.layout);
    document.dispatchEvent(new CustomEvent('ff:split', { detail: { ratio: s.ratio } }));
    for (const [id, sel] of DRAWERS) if (!s.drawers.includes(id) && expanded(sel)) toggleOf(sel)?.click();
    for (const [id, sel] of DRAWERS) if (s.drawers.includes(id) && !expanded(sel)) toggleOf(sel)?.click();
    const html = document.documentElement;
    const open = html.getAttribute('data-ft-dock') === 'open';
    if (s.dock.open) {
      html.setAttribute('data-ft-dock-h', String(s.dock.h));
      html.style.setProperty('--ft-dock-size', `${s.dock.h}px`);
    }
    if (open !== s.dock.open) document.dispatchEvent(new CustomEvent('ff:dock-toggle', { detail: { kind: 'shell' } }));
    this.remember(name);
    return true;
  }

  private remember(name: string | null): void {
    try { if (name) localStorage.setItem(NAME_KEY, name); else localStorage.removeItem(NAME_KEY); } catch { /* this session only */ }
    this.emit();
  }

  private emit(): void { for (const fn of this.listeners) fn(); }
}
