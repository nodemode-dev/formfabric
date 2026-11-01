import { call } from '../../engine/service';
import { setClass, setHidden, setText } from '../../ui/dom';
import { t, onLanguage } from '../../ui/i18n';
import { toast } from '../../ui/toast';
import type { PanelStateJson } from './types';
import { button, ensureStyle, labelled, sliderField, type Field } from './ui';

const STIFF_MIN = -2;
const STIFF_MAX = 3;

function fmtFactor(v: number): string {
  return `x${v >= 100 ? v.toFixed(0) : v >= 10 ? v.toFixed(1) : v.toFixed(2)}`;
}

export class PanelStrip {
  readonly root: HTMLElement;
  private ids: number[] = [];
  private enabled = false;
  private state: PanelStateJson = {};
  private readonly count: HTMLElement;
  private readonly freeze: HTMLButtonElement;
  private readonly disable: HTMLButtonElement;
  private readonly hold: HTMLButtonElement;
  private readonly stiffen: Field<number>;
  private readonly weft: Field<number>;
  private readonly warp: Field<number>;

  constructor(parent: HTMLElement) {
    ensureStyle();
    const root = document.createElement('div');
    root.className = 'ffd-strip';
    root.hidden = true;
    this.root = root;
    const title = document.createElement('span');
    labelled(title, 'Panel state');
    this.count = document.createElement('span');
    this.freeze = button('Freeze', () => this.send({ frozen: !this.state.frozen }));
    this.disable = button('Disable', () => this.send({ disabled: !this.state.disabled }));
    this.hold = button('Hold shape', () => this.send({ hold_shape: !this.state.hold_shape }));
    this.stiffen = sliderField({
      english: 'Stiffen', min: STIFF_MIN, max: STIFF_MAX, step: 0.05, value: 1, fmt: fmtFactor,
      toValue: (r) => Math.pow(10, r), toRaw: (v) => Math.log10(Math.max(v, 0.01)),
      commit: (v) => this.send({ stiffen: Math.round(v * 1000) / 1000 }),
    });
    this.weft = sliderField({ english: 'Shrink weft', min: 0.3, max: 2, step: 0.01, value: 1, fmt: (v) => v.toFixed(2), commit: (v) => this.send({ shrink_weft: v }) });
    this.warp = sliderField({ english: 'Shrink warp', min: 0.3, max: 2, step: 0.01, value: 1, fmt: (v) => v.toFixed(2), commit: (v) => this.send({ shrink_warp: v }) });
    const reset = button('Reset', () => this.send({ frozen: false, disabled: false, hold_shape: false, stiffen: 1, shrink_weft: 1, shrink_warp: 1 }));
    root.append(title, this.count, this.freeze, this.disable, this.hold, this.stiffen.root, this.weft.root, this.warp.root, reset);
    parent.appendChild(root);
    onLanguage(() => this.paintCount());
  }

  setEnabled(on: boolean): void {
    this.enabled = on;
    this.paintVisible();
  }

  select(ids: number[]): void {
    this.ids = ids;
    this.paintVisible();
    this.paintCount();
  }

  sync(state: PanelStateJson | null): void {
    this.state = { ...(state ?? {}) };
    const s = this.state;
    setClass(this.freeze, 'is-on', s.frozen === true);
    setClass(this.disable, 'is-on', s.disabled === true);
    setClass(this.hold, 'is-on', s.hold_shape === true);
    this.stiffen.set(s.stiffen ?? 1);
    this.weft.set(s.shrink_weft ?? 1);
    this.warp.set(s.shrink_warp ?? 1);
  }

  private paintVisible(): void {
    setHidden(this.root, !this.enabled || this.ids.length === 0);
  }

  private paintCount(): void {
    setText(this.count, this.ids.length > 1 ? `${this.ids.length} ${t('panels')}` : '');
  }

  private send(args: PanelStateJson): void {
    if (this.ids.length === 0) return;
    Object.assign(this.state, args);
    call('panel.state', { panels: this.ids, ...args }).catch((err: unknown) => toast(err instanceof Error ? err.message : String(err), 'error'));
  }
}
