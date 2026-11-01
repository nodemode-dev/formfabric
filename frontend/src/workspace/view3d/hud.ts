import { setClass, setText } from '../../ui/dom';
import { onLanguage, t } from '../../ui/i18n';
import type { ViewStyle } from './model';

export type Preset = 'front' | 'back' | 'left' | 'right' | 'top' | 'all';

const CSS = `
.ff3-bar{position:absolute;left:8px;top:8px;z-index:var(--z-raised);display:flex;flex-wrap:wrap;gap:6px;max-width:calc(100% - 16px);pointer-events:none}
.ff3-seg{display:flex;pointer-events:auto;overflow:hidden;border:1px solid var(--border-color);border-radius:var(--radius-sm);background:color-mix(in srgb,var(--bg-secondary) 88%,transparent)}
.ff3-btn{height:22px;min-width:24px;padding:0 7px;border:0;background:transparent;color:var(--text-secondary);font:500 var(--fs-11)/1 var(--font-ui);cursor:pointer}
.ff3-btn+.ff3-btn{border-left:1px solid var(--border-color)}
.ff3-btn:hover{background:var(--bg-hover);color:var(--text-primary)}
.ff3-btn.is-on{background:var(--accent-color-dim);color:var(--accent-color)}
.ff3-overlay{position:absolute;left:0;top:0;pointer-events:none}
`;

const STYLES: Array<[ViewStyle, string, string]> = [
  ['shaded', 'Shaded', 'Shaded'],
  ['shaded-mesh', 'Shaded + mesh', 'Shaded with mesh'],
  ['mesh', 'Mesh', 'Mesh only'],
  ['panels', 'Colors', 'Panel colors'],
];

const PRESETS: Array<[Preset, string, string]> = [
  ['front', '1', 'Front'],
  ['back', '2', 'Back'],
  ['left', '3', 'Left'],
  ['right', '4', 'Right'],
  ['top', '5', 'Top'],
  ['all', '0', 'Frame all'],
];

export interface HudHandlers {
  style(s: ViewStyle): void;
  preset(p: Preset): void;
  stats(on: boolean): void;
}

export class Hud {
  readonly overlay: HTMLCanvasElement;
  readonly ctx: CanvasRenderingContext2D;
  private readonly styleButtons = new Map<ViewStyle, HTMLButtonElement>();
  private readonly presetButtons: Array<[Preset, HTMLButtonElement]> = [];
  private readonly statsButton: HTMLButtonElement;

  constructor(pane: HTMLElement, private readonly handlers: HudHandlers, style: ViewStyle, stats: boolean) {
    if (!document.getElementById('ff3-style')) {
      const el = document.createElement('style');
      el.id = 'ff3-style';
      el.textContent = CSS;
      document.head.appendChild(el);
    }
    this.overlay = document.createElement('canvas');
    this.overlay.className = 'ff3-overlay';
    this.ctx = this.overlay.getContext('2d') as CanvasRenderingContext2D;
    const bar = document.createElement('div');
    bar.className = 'ff3-bar';
    const styles = document.createElement('div');
    styles.className = 'ff3-seg';
    for (const [id, label] of STYLES) {
      const b = this.button(label, () => handlers.style(id));
      styles.appendChild(b);
      this.styleButtons.set(id, b);
    }
    const views = document.createElement('div');
    views.className = 'ff3-seg';
    for (const [id, key] of PRESETS) {
      const b = this.button(key, () => handlers.preset(id));
      views.appendChild(b);
      this.presetButtons.push([id, b]);
    }
    const info = document.createElement('div');
    info.className = 'ff3-seg';
    this.statsButton = this.button('Stats', () => handlers.stats(!this.statsButton.classList.contains('is-on')));
    info.appendChild(this.statsButton);
    bar.append(styles, views, info);
    pane.append(this.overlay, bar);
    this.setStyle(style);
    this.setStats(stats);
    this.labels();
    onLanguage(() => this.labels());
  }

  private button(text: string, onClick: () => void): HTMLButtonElement {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'ff3-btn';
    b.textContent = text;
    b.addEventListener('click', () => { onClick(); b.blur(); });
    return b;
  }

  private labels(): void {
    for (const [id, label, title] of STYLES) {
      const b = this.styleButtons.get(id) as HTMLButtonElement;
      setText(b, t(label));
      if (b.title !== t(title)) b.title = t(title);
    }
    for (const [id, key, name] of PRESETS) {
      const b = this.presetButtons.find((p) => p[0] === id)?.[1] as HTMLButtonElement;
      const title = `${t(name)} (${key})`;
      if (b.title !== title) b.title = title;
    }
    setText(this.statsButton, t('Stats'));
    const title = t('Statistics');
    if (this.statsButton.title !== title) this.statsButton.title = title;
  }

  setStyle(s: ViewStyle): void {
    for (const [id, b] of this.styleButtons) setClass(b, 'is-on', id === s);
  }

  setStats(on: boolean): void {
    setClass(this.statsButton, 'is-on', on);
  }

  resize(w: number, h: number, dpr: number): void {
    const bw = Math.round(w * dpr);
    const bh = Math.round(h * dpr);
    if (this.overlay.width !== bw) this.overlay.width = bw;
    if (this.overlay.height !== bh) this.overlay.height = bh;
    this.overlay.style.width = `${w}px`;
    this.overlay.style.height = `${h}px`;
  }
}
