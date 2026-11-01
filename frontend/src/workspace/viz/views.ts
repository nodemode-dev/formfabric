import { patchList, setDisabled, setText, setTitle } from '../../ui/dom';
import { promptName } from '../../ui/dialogs';
import { t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import type { CameraPose } from '../view3d/camera';
import { hasCommand, missing, num, obj, run } from './api';
import type { VizHost } from './host';
import { Pop } from './pop';
import type { SavedViewJson } from './types';

const MOVE_MS = 520;

function poseOf(v: unknown): CameraPose | null {
  const o = obj(v);
  const target = o.target;
  if (!Array.isArray(target) || target.length < 3) return null;
  const distance = num(o.distance);
  const yaw = num(o.yaw);
  const pitch = num(o.pitch);
  if (distance === null || yaw === null || pitch === null) return null;
  return { target: [Number(target[0]), Number(target[1]), Number(target[2])], distance, yaw, pitch, ortho: o.ortho === true };
}

function ease(x: number): number {
  return x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2;
}

export class Views {
  private readonly list: HTMLDivElement;
  private readonly save: HTMLButtonElement;
  private readonly empty: HTMLElement;
  private raf = 0;
  private items: SavedViewJson[] = [];

  constructor(private readonly host: VizHost, private readonly pop: Pop) {
    pop.setLabel('Views');
    this.save = pop.item('Save current view', () => void this.saveCurrent(), 'add');
    pop.separator();
    this.list = document.createElement('div');
    this.list.className = 'ffz-rows';
    pop.panel.appendChild(this.list);
    this.empty = pop.note('No saved views');
    pop.onOpen = () => this.update();
  }

  available(): void {
    const ok = hasCommand('view.save');
    setDisabled(this.save, !ok);
    setTitle(this.pop.button, ok ? t('Saved views') : missing('view.save'));
  }

  update(): void {
    this.items = this.host.viz().views;
    setText(this.empty, this.items.length ? '' : t('No saved views'));
    this.empty.hidden = this.items.length > 0;
    patchList(this.list, this.items, {
      key: (v) => String(v.id),
      create: (v) => this.row(v),
      update: (node, v) => {
        const name = node.querySelector('.ffz-grow') as HTMLElement;
        setText(name, v.name);
        node.dataset.id = String(v.id);
      },
    });
  }

  private row(v: SavedViewJson): HTMLElement {
    const row = document.createElement('div');
    row.className = 'ffz-row';
    row.dataset.id = String(v.id);
    row.innerHTML = `<button class="ffz-item" type="button"><span class="ffz-grow"></span></button><button class="cv-icon-btn" type="button" data-k="rename">${icon('pen', 12)}</button><button class="cv-icon-btn" type="button" data-k="del">${icon('delete', 12)}</button>`;
    const [apply, rename, del] = Array.from(row.querySelectorAll('button'));
    rename.title = t('Rename');
    del.title = t('Delete');
    apply.addEventListener('click', () => { const cur = this.find(row.dataset.id); if (cur) this.apply(cur); apply.blur(); });
    rename.addEventListener('click', () => { const cur = this.find(row.dataset.id); if (cur) void this.rename(cur); rename.blur(); });
    del.addEventListener('click', () => { const cur = this.find(row.dataset.id); if (cur) void run('view.remove', { id: cur.id }); del.blur(); });
    return row;
  }

  private find(id: string | undefined): SavedViewJson | undefined {
    return this.items.find((v) => String(v.id) === id);
  }

  private async saveCurrent(): Promise<void> {
    const fallback = `${t('View')} ${this.items.length + 1}`;
    const name = await promptName(t('Save current view'), fallback, t('Save'));
    if (!name) return;
    await run('view.save', { name, camera: this.host.camera.pose() });
  }

  private async rename(v: SavedViewJson): Promise<void> {
    const name = await promptName(t('Rename'), v.name, t('Rename'));
    if (!name || name === v.name) return;
    const saved = await run('view.save', { name, camera: v.camera });
    if (saved !== undefined) await run('view.remove', { id: v.id });
  }

  apply(v: SavedViewJson): void {
    const to = poseOf(v.camera);
    if (!to) return;
    const cam = this.host.camera;
    const from = cam.pose();
    this.host.setAutoFrame(false);
    cancelAnimationFrame(this.raf);
    let dyaw = to.yaw - from.yaw;
    dyaw = ((((dyaw + 180) % 360) + 360) % 360) - 180;
    const t0 = performance.now();
    const step = (now: number): void => {
      const k = Math.min(1, (now - t0) / MOVE_MS);
      const e = ease(k);
      cam.setPose({
        target: [from.target[0] + (to.target[0] - from.target[0]) * e, from.target[1] + (to.target[1] - from.target[1]) * e, from.target[2] + (to.target[2] - from.target[2]) * e],
        distance: from.distance * Math.pow(to.distance / from.distance, e),
        yaw: from.yaw + dyaw * e,
        pitch: from.pitch + (to.pitch - from.pitch) * e,
        ortho: k >= 1 ? to.ortho : from.ortho,
      });
      this.host.redraw();
      if (k < 1) this.raf = requestAnimationFrame(step);
    };
    this.raf = requestAnimationFrame(step);
    this.pop.close();
  }
}
