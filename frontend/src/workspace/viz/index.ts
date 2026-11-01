import type { ScalarsFrame } from '../../engine/stream';
import { onEngineChanged } from '../../engine/service';
import { promptName } from '../../ui/dialogs';
import { setDisabled, setTitle } from '../../ui/dom';
import { icon } from '../../ui/icons';
import { onLanguage, t } from '../../ui/i18n';
import type { Frame } from '../view3d/frame';
import type { Scene } from '../view3d/scene';
import { hasCommand, missing, onCommands, refreshCommands, run } from './api';
import { Capture } from './capture';
import type { VizHost, WorldHit } from './host';
import { Maps } from './maps';
import { Measure } from './measure';
import { Pop } from './pop';
import { StageDrawer } from './stage';
import { Views } from './views';
import './viz.css';

export type { VizHost, WorldHit } from './host';

const SCHEMATIC_KEY = 'formfabric.schematic.path';

function button(group: HTMLElement, iconName: string, english: string): HTMLButtonElement {
  const slot = document.createElement('div');
  slot.className = 'ffz-slot';
  const b = document.createElement('button');
  b.type = 'button';
  b.className = 'ff3-btn';
  b.innerHTML = `${icon(iconName, 14)}<span></span>`;
  const span = b.querySelector('span:not(.icon)') as HTMLElement;
  const paint = (): void => { const s = t(english); if (span.textContent !== s) span.textContent = s; };
  paint();
  onLanguage(paint);
  slot.appendChild(b);
  group.appendChild(slot);
  return b;
}

export class VizUi {
  readonly maps: Maps;
  readonly views: Views;
  readonly measure: Measure;
  private readonly capture: Capture;
  private readonly stage: StageDrawer;
  private readonly stageButton: HTMLButtonElement;
  private readonly captureButton: HTMLButtonElement;
  private readonly schematic: HTMLButtonElement;

  constructor(private readonly host: VizHost) {
    const group = document.createElement('div');
    group.className = 'ffz-group';
    host.bar.appendChild(group);
    this.maps = new Maps(host, new Pop(group, 'layers', 'Maps'));
    this.views = new Views(host, new Pop(group, 'eye', 'Views'));
    this.measure = new Measure(host, new Pop(group, 'ruler', 'Measure'));
    this.stageButton = button(group, 'sun', 'Stage');
    this.captureButton = button(group, 'image', 'Capture');
    this.schematic = button(group, 'download', 'Schematic');
    this.stage = new StageDrawer(host, this.stageButton);
    this.capture = new Capture(host);
    this.captureButton.addEventListener('click', () => { this.capture.open(); this.captureButton.blur(); });
    this.schematic.addEventListener('click', () => { void this.exportSchematic(); this.schematic.blur(); });
    setTitle(this.stageButton, t('Lights, environment and ground'));
    setTitle(this.captureButton, t('Snapshot, video and turntable'));
    onCommands(() => this.available());
    onEngineChanged((e) => this.measure.changed(e.name));
    void refreshCommands();
    void this.measure.pull();
    this.docChanged();
  }

  private available(): void {
    this.maps.available();
    this.views.available();
    this.measure.available();
    this.stage.available();
    const ok = hasCommand('schematic.export');
    setDisabled(this.schematic, !ok);
    setTitle(this.schematic, ok ? t('Export a schematic drawing of the garment') : missing('schematic.export'));
    this.host.redraw();
  }

  docChanged(): void {
    const v = this.host.viz();
    this.maps.sync(v.map);
    this.views.update();
    this.stage.refresh();
  }

  hdriStatus(state: string): void {
    this.stage.setHdriStatus(state);
  }

  onScalars(f: ScalarsFrame): void {
    this.maps.onScalars(f);
  }

  click(hit: WorldHit | null): boolean {
    if (!this.measure.active) return false;
    this.measure.click(hit);
    return true;
  }

  draw(ctx: CanvasRenderingContext2D, f: Frame, scene: Scene, dpr: number, accent: string, ring: string): void {
    this.measure.draw(ctx, f, scene, dpr, accent, ring);
  }

  private async exportSchematic(): Promise<void> {
    const path = await promptName(t('Export a schematic drawing of the garment'), localStorage.getItem(SCHEMATIC_KEY) ?? '', t('Export'));
    if (!path) return;
    const lower = path.toLowerCase();
    const format = lower.endsWith('.pdf') ? 'pdf' : 'svg';
    if (!lower.endsWith(`.${format}`)) {
      await this.export(`${path}.svg`, 'svg');
      return;
    }
    await this.export(path, format);
  }

  private async export(path: string, format: 'svg' | 'pdf'): Promise<void> {
    localStorage.setItem(SCHEMATIC_KEY, path);
    await run('schematic.export', { path, format });
  }
}
