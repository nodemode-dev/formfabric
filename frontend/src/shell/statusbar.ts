import { pendingClass, q, setClass, setText, setTitle, type Disposer } from '../ui/dom';
import { icon } from '../ui/icons';
import { onLanguage, t } from '../ui/i18n';
import type { Link } from '../engine/service';

export class StatusBar {
  private readonly units: HTMLButtonElement;
  private readonly panels: HTMLElement;
  private readonly engine: HTMLElement;
  private readonly gpu: HTMLElement;
  private readonly hint: HTMLElement;
  private readonly busy: HTMLElement;
  private hintSegs: string[] = [];
  private hintTimer = 0;
  private version = '';
  private link: Link = 'connecting';
  private gpuName = '';
  private count: number | null = null;
  onUnits: (() => void) | null = null;

  constructor(root: HTMLElement) {
    root.insertAdjacentHTML('beforeend',
      '<button class="cv-st-units" type="button" title="Units: click to change" aria-label="Units"></button>' +
      '<span class="cv-st-panels"></span>' +
      '<span class="cv-st-engine"></span><span class="cv-st-gpu"></span>' +
      '<span class="cv-st-hint"></span>' +
      `<span class="cv-st-busy">${icon('loading', 12)}<span class="cv-st-busy-text"></span></span>`);
    this.units = q(root, '.cv-st-units');
    this.panels = q(root, '.cv-st-panels');
    this.engine = q(root, '.cv-st-engine');
    this.gpu = q(root, '.cv-st-gpu');
    this.hint = q(root, '.cv-st-hint');
    this.busy = q(root, '.cv-st-busy');
    this.units.addEventListener('click', () => this.onUnits?.());
    this.setUnits('mm');
    this.setPanels(null);
    this.paintEngine();
    onLanguage(() => { this.paintEngine(); this.paintPanels(); this.paintGpu(); });
  }

  setUnits(u: string): void { setText(this.units, u); }

  setPanels(n: number | null): void { this.count = n; this.paintPanels(); }

  private paintPanels(): void {
    setText(this.panels, this.count === null ? `${t('Panels')}: -` : `${t('Panels')}: ${this.count}`);
  }

  setEngine(link: Link, version = this.version): void {
    this.link = link;
    this.version = version || this.version;
    this.paintEngine();
  }

  private paintEngine(): void {
    const up = this.link === 'up';
    setText(this.engine, up ? `${t('Engine')} ${this.version}${'__TAURI_INTERNALS__' in window ? ' · desktop' : ''}`.trim() : this.link === 'connecting' ? t('Engine starting') : t('Engine offline'));
    setClass(this.engine, 'is-down', this.link === 'down');
    setTitle(this.engine, up ? '' : t('The engine is not running. Start it with ./formfabric'));
  }

  setGpu(name: string): void { this.gpuName = name; this.paintGpu(); }

  private paintGpu(): void { setText(this.gpu, this.gpuName ? `${t('GPU')}: ${this.gpuName}` : ''); }

  setHint(text: string, error = false): void {
    window.clearTimeout(this.hintTimer);
    this.hintSegs = text ? text.split(' · ') : [];
    setText(this.hint, this.hintSegs.map(t).join(' · '));
    setClass(this.hint, 'is-error', error);
    if (error && text) this.hintTimer = window.setTimeout(() => this.setHint(''), 6000);
  }

  working(label: string): Disposer {
    setText(q(this.busy, '.cv-st-busy-text'), t(label));
    return pendingClass(this.busy);
  }
}
