import { Dialog } from '../../ui/dialogs';
import { el, setDisabled, setHidden, setText } from '../../ui/dom';
import { t } from '../../ui/i18n';
import { toast } from '../../ui/toast';
import { CAPTURE_VIEWS, type PresetName } from '../view3d/camera';
import { hasCommand, message, num, obj, quiet, run } from './api';
import { encodeGif } from './gif';
import type { VizHost } from './host';
import { withPpi } from './png';

type Kind = 'snapshot' | 'multi' | 'video' | 'turntable' | 'gif';

const PATH_KEY = 'formfabric.capture.path';
const FPS = 30;
const GIF_FPS = 20;
const GIF_SIDE = 640;
const GIF_FRAMES = 240;
const MAX_SIDE = 16384;
const MAX_VIEWS = 10;
const DEFAULT_PPI = 72;
const DEFAULT_SLOTS: PresetName[] = ['front', 'back', 'left', 'right', 'left34', 'right34', 'top', 'bottom', 'front', 'back'];
const SIZES: Array<[string, string, number, number]> = [
  ['viewport', 'Viewport size', 0, 0],
  ['720', '1280 x 720', 1280, 720],
  ['1080', '1920 x 1080', 1920, 1080],
  ['2160', '3840 x 2160', 3840, 2160],
  ['custom', 'Custom', 0, 0],
];

function toBase64(bytes: Uint8Array): string {
  let s = '';
  const step = 0x8000;
  for (let i = 0; i < bytes.length; i += step) s += String.fromCharCode(...bytes.subarray(i, i + step));
  return btoa(s);
}

function wait(ms: number): Promise<void> {
  return new Promise((r) => window.setTimeout(r, ms));
}

export class Capture {
  private dialog: Dialog | null = null;
  private kind: HTMLSelectElement | null = null;
  private size: HTMLSelectElement | null = null;
  private w: HTMLInputElement | null = null;
  private h: HTMLInputElement | null = null;
  private transparent: HTMLInputElement | null = null;
  private ppi: HTMLInputElement | null = null;
  private count: HTMLInputElement | null = null;
  private combine: HTMLSelectElement | null = null;
  private separate: HTMLInputElement | null = null;
  private slots: HTMLSelectElement[] = [];
  private slotRows: HTMLElement[] = [];
  private seconds: HTMLInputElement | null = null;
  private range: HTMLInputElement | null = null;
  private path: HTMLInputElement | null = null;
  private status: HTMLElement | null = null;
  private go: HTMLButtonElement | null = null;
  private rows: Record<string, HTMLElement> = {};
  private busy = false;
  private cancel = false;

  constructor(private readonly host: VizHost) {}

  private build(): Dialog {
    const d = new Dialog('Capture', 420);
    if (!document.getElementById('overlay-root')) document.body.appendChild(d.scrim);
    const body = el('<div class="ffz-cap"></div>');
    d.body.appendChild(body);
    const kind = el<HTMLSelectElement>('<select class="cv-input"><option value="snapshot">Snapshot (PNG)</option><option value="multi">Multi-view snapshot (PNG)</option><option value="video">Video (WebM)</option><option value="turntable">Turntable (WebM)</option><option value="gif">Turntable (GIF)</option></select>');
    for (const o of Array.from(kind.options)) o.textContent = t(o.textContent ?? '');
    const size = el<HTMLSelectElement>('<select class="cv-input"></select>');
    for (const [v, label] of SIZES) {
      const o = document.createElement('option');
      o.value = v;
      o.textContent = t(label);
      size.appendChild(o);
    }
    const dims = el('<div class="ffz-cap-size"><input class="cv-input" type="number" min="16" max="8192" step="1" value="1920"><input class="cv-input" type="number" min="16" max="8192" step="1" value="1080"></div>');
    const [w, h] = Array.from(dims.querySelectorAll('input'));
    const transparent = el<HTMLInputElement>('<input type="checkbox">');
    const ppi = el<HTMLInputElement>(`<input class="cv-input" type="number" min="1" max="4800" step="1" value="${DEFAULT_PPI}">`);
    const count = el<HTMLInputElement>(`<input class="cv-input" type="number" min="2" max="${MAX_VIEWS}" step="1" value="4">`);
    const combine = el<HTMLSelectElement>('<select class="cv-input"><option value="h">Horizontal</option><option value="v">Vertical</option></select>');
    for (const o of Array.from(combine.options)) o.textContent = t(o.textContent ?? '');
    const separate = el<HTMLInputElement>('<input type="checkbox">');
    const seconds = el<HTMLInputElement>('<input class="cv-input" type="number" min="1" max="120" step="1" value="4">');
    const range = el<HTMLInputElement>('<input type="checkbox">');
    const path = el<HTMLInputElement>('<input class="cv-input" type="text" spellcheck="false" placeholder="/absolute/path/capture.png">');
    path.value = localStorage.getItem(PATH_KEY) ?? '';
    this.rows.kind = d.field('Type', kind);
    this.rows.size = d.field('Size', size);
    this.rows.dims = d.field('Width and height', dims);
    this.rows.transparent = d.field('Transparent background', transparent);
    this.rows.ppi = d.field(t('Resolution (ppi)'), ppi);
    this.rows.count = d.field(t('Number of views'), count);
    this.rows.combine = d.field(t('Combine'), combine);
    this.rows.separate = d.field(t('Save separate images'), separate);
    this.slots = [];
    this.slotRows = [];
    for (let i = 0; i < MAX_VIEWS; i++) {
      const s = el<HTMLSelectElement>('<select class="cv-input"></select>');
      for (const [v, name] of CAPTURE_VIEWS) {
        const o = document.createElement('option');
        o.value = v;
        o.textContent = t(name);
        s.appendChild(o);
      }
      s.value = DEFAULT_SLOTS[i];
      s.dataset.k = `cap-view-${i}`;
      this.slots.push(s);
      this.slotRows.push(d.field(`${t('View')} ${i + 1}`, s));
    }
    this.rows.seconds = d.field('Seconds', seconds);
    this.rows.range = d.field('Use the timeline range', range);
    this.rows.path = d.field('Save to', path);
    const status = el('<p class="ffz-cap-status"></p>');
    d.body.appendChild(status);
    d.button('Close', 'ghost', () => { if (this.busy) this.cancel = true; else d.close(); });
    this.go = d.button('Capture', 'primary', () => void this.start());
    Object.assign(this, { kind, size, w, h, transparent, ppi, count, combine, separate, seconds, range, path, status });
    kind.dataset.k = 'cap-kind';
    ppi.dataset.k = 'cap-ppi';
    count.dataset.k = 'cap-count';
    combine.dataset.k = 'cap-combine';
    separate.dataset.k = 'cap-separate';
    transparent.dataset.k = 'cap-transparent';
    count.addEventListener('change', () => this.layout());
    size.dataset.k = 'cap-size';
    seconds.dataset.k = 'cap-seconds';
    path.dataset.k = 'cap-path';
    status.dataset.k = 'cap-status';
    this.go.dataset.k = 'cap-go';
    kind.addEventListener('change', () => this.layout());
    size.addEventListener('change', () => this.layout());
    path.addEventListener('change', () => localStorage.setItem(PATH_KEY, path.value.trim()));
    this.dialog = d;
    this.layout();
    return d;
  }

  open(): void {
    const d = this.dialog ?? this.build();
    this.layout();
    d.open();
  }

  private layout(): void {
    if (!this.kind || !this.size) return;
    const kind = this.kind.value as Kind;
    const multi = kind === 'multi';
    const snap = kind === 'snapshot' || multi;
    setHidden(this.rows.size, !snap);
    setHidden(this.rows.dims, !snap || this.size.value !== 'custom');
    setHidden(this.rows.transparent, !(snap || kind === 'gif'));
    setHidden(this.rows.ppi, !snap);
    setHidden(this.rows.count, !multi);
    setHidden(this.rows.combine, !multi);
    setHidden(this.rows.separate, !multi);
    const n = this.viewCount();
    this.slotRows.forEach((row, i) => setHidden(row, !multi || i >= n));
    setHidden(this.rows.seconds, snap);
    setHidden(this.rows.range, kind !== 'video' || !hasCommand('anim.get'));
    const ext = snap ? 'png' : kind === 'gif' ? 'gif' : 'webm';
    if (this.path && this.path.value && !this.path.value.endsWith(`.${ext}`)) this.path.value = this.path.value.replace(/\.[A-Za-z0-9]+$/, `.${ext}`);
    if (this.path) this.path.placeholder = `/absolute/path/capture.${ext}`;
  }

  private say(text: string): void {
    setText(this.status, text);
  }

  private async start(): Promise<void> {
    if (this.busy || !this.kind || !this.path || !this.go) return;
    const kind = this.kind.value as Kind;
    const target = this.path.value.trim();
    const canWrite = hasCommand('capture.write');
    if (canWrite && !target.startsWith('/') && !/^[A-Za-z]:[\\/]/.test(target)) {
      this.say(t('Enter an absolute path to save to'));
      return;
    }
    localStorage.setItem(PATH_KEY, target);
    this.busy = true;
    this.cancel = false;
    setDisabled(this.go, true);
    try {
      if (kind === 'multi' && this.separate?.checked) { await this.multiSeparate(target, canWrite); return; }
      const blob = kind === 'snapshot' ? await this.snapshot() : kind === 'multi' ? await this.multi() : kind === 'video' ? await this.video() : kind === 'gif' ? await this.gif() : await this.turntable();
      if (!blob) return;
      await this.deliver(blob, target, canWrite, kind === 'snapshot' || kind === 'multi' ? 'png' : kind === 'gif' ? 'gif' : 'webm');
    } catch (err) {
      this.say(message(err));
      toast(message(err), 'error');
    } finally {
      this.busy = false;
      this.host.redraw();
      setDisabled(this.go, false);
    }
  }

  private async deliver(blob: Blob, target: string, canWrite: boolean, ext: string, name = 'formfabric-capture'): Promise<void> {
    if (canWrite) {
      this.say(t('Writing the file'));
      const data = toBase64(new Uint8Array(await blob.arrayBuffer()));
      const r = await run<unknown>('capture.write', { path: target, data, kind: ext });
      if (r === undefined) { this.say(t('The engine refused the file')); return; }
      this.say(`${t('Saved')} ${String(obj(r).path ?? target)} (${blob.size} B)`);
      toast(`${t('Saved')} ${target}`, 'info');
      return;
    }
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `${name}.${ext}`;
    document.body.appendChild(a);
    a.click();
    a.remove();
    window.setTimeout(() => URL.revokeObjectURL(a.href), 4000);
    this.say(`${t('Downloaded')} ${name}.${ext} (${blob.size} B)`);
  }

  private dims(): [number, number] {
    const size = this.size?.value ?? 'viewport';
    let w = this.host.canvas.width;
    let h = this.host.canvas.height;
    if (size === 'custom') {
      w = Math.round(Number(this.w?.value));
      h = Math.round(Number(this.h?.value));
    } else if (size !== 'viewport') {
      const s = SIZES.find((x) => x[0] === size);
      if (s) { w = s[2]; h = s[3]; }
    }
    const max = this.host.maxSize();
    if (!(w >= 16 && h >= 16)) throw new Error(t('The size is not valid'));
    if (w > max || h > max) throw new Error(`${t('The graphics card limit is')} ${max} px`);
    return [w, h];
  }

  private async png(canvas: HTMLCanvasElement): Promise<Blob> {
    const blob = await new Promise<Blob | null>((res) => canvas.toBlob(res, 'image/png'));
    if (!blob) throw new Error(t('The image could not be encoded'));
    const ppi = Math.max(1, Math.min(4800, Math.round(Number(this.ppi?.value)) || DEFAULT_PPI));
    return new Blob([withPpi(new Uint8Array(await blob.arrayBuffer()), ppi)], { type: 'image/png' });
  }

  private async snapshot(): Promise<Blob | null> {
    const [w, h] = this.dims();
    this.say(t('Rendering'));
    await wait(30);
    const canvas = this.host.snapshot(w, h, !!this.transparent?.checked);
    if (!canvas) throw new Error(t('Rendering is not available'));
    return this.png(canvas);
  }

  private viewCount(): number {
    return Math.max(2, Math.min(MAX_VIEWS, Math.round(Number(this.count?.value)) || 2));
  }

  private async views(): Promise<Array<{ name: PresetName; canvas: HTMLCanvasElement }>> {
    const [w, h] = this.dims();
    const n = this.viewCount();
    const cam = this.host.camera;
    const pose = cam.pose();
    const out: Array<{ name: PresetName; canvas: HTMLCanvasElement }> = [];
    this.host.setAutoFrame(false);
    try {
      for (let i = 0; i < n; i++) {
        const name = (this.slots[i]?.value ?? DEFAULT_SLOTS[i]) as PresetName;
        this.say(`${t('Rendering')} ${i + 1} / ${n}`);
        cam.snap(name);
        await wait(20);
        const canvas = this.host.snapshot(w, h, !!this.transparent?.checked);
        if (!canvas) throw new Error(t('Rendering is not available'));
        out.push({ name, canvas });
      }
    } finally {
      cam.setPose(pose);
      this.host.redraw();
    }
    return out;
  }

  private async multi(): Promise<Blob | null> {
    const list = await this.views();
    const vertical = this.combine?.value === 'v';
    const w = list[0].canvas.width;
    const h = list[0].canvas.height;
    const total = (vertical ? h : w) * list.length;
    if (total > MAX_SIDE) throw new Error(`${t('The combined image is too large')} (${total} px)`);
    const sheet = document.createElement('canvas');
    sheet.width = vertical ? w : w * list.length;
    sheet.height = vertical ? h * list.length : h;
    const ctx = sheet.getContext('2d') as CanvasRenderingContext2D;
    list.forEach((v, i) => ctx.drawImage(v.canvas, vertical ? 0 : i * w, vertical ? i * h : 0));
    return this.png(sheet);
  }

  private async multiSeparate(target: string, canWrite: boolean): Promise<void> {
    const list = await this.views();
    const stem = target.replace(/\.png$/i, '');
    let at = 0;
    for (const v of list) {
      at++;
      const name = `${at}-${v.name}`;
      await this.deliver(await this.png(v.canvas), `${stem}-${name}.png`, canWrite, 'png', `formfabric-capture-${name}`);
    }
  }

  private async gif(): Promise<Blob | null> {
    const secs = Math.max(1, Math.min(120, Number(this.seconds?.value) || 4));
    const frames = Math.max(2, Math.min(GIF_FRAMES, Math.round(secs * GIF_FPS)));
    const k = Math.min(1, GIF_SIDE / Math.max(this.host.canvas.width, this.host.canvas.height, 1));
    const w = Math.max(16, Math.round(this.host.canvas.width * k));
    const h = Math.max(16, Math.round(this.host.canvas.height * k));
    const alpha = !!this.transparent?.checked;
    const cam = this.host.camera;
    const yaw = cam.yaw;
    this.host.setAutoFrame(false);
    const images: ImageData[] = [];
    try {
      for (let i = 0; i < frames && !this.cancel; i++) {
        cam.yaw = yaw - (i / frames) * Math.PI * 2;
        const canvas = this.host.snapshot(w, h, alpha);
        if (!canvas) throw new Error(t('Rendering is not available'));
        images.push((canvas.getContext('2d') as CanvasRenderingContext2D).getImageData(0, 0, w, h));
        this.say(`${t('Recording')} ${i + 1} / ${frames}`);
        await wait(0);
      }
    } finally {
      cam.yaw = yaw;
      this.host.redraw();
    }
    if (images.length < 2) return null;
    this.say(t('Encoding'));
    await wait(0);
    return new Blob([encodeGif(images, w, h, Math.round(100 / GIF_FPS), alpha)], { type: 'image/gif' });
  }

  private recorder(): { rec: MediaRecorder; done: Promise<Blob>; push: () => void } {
    if (typeof MediaRecorder === 'undefined') throw new Error(t('This window cannot record video'));
    const stream = this.host.canvas.captureStream(FPS);
    const types = ['video/webm;codecs=vp9', 'video/webm;codecs=vp8', 'video/webm'];
    const mime = types.find((m) => MediaRecorder.isTypeSupported(m));
    const rec = new MediaRecorder(stream, mime ? { mimeType: mime, videoBitsPerSecond: 8_000_000 } : undefined);
    const chunks: Blob[] = [];
    rec.ondataavailable = (e) => { if (e.data.size) chunks.push(e.data); };
    const done = new Promise<Blob>((res) => { rec.onstop = () => res(new Blob(chunks, { type: 'video/webm' })); });
    const track = stream.getVideoTracks()[0] as (MediaStreamTrack & { requestFrame?: () => void }) | undefined;
    return { rec, done, push: () => track?.requestFrame?.() };
  }

  private async video(): Promise<Blob | null> {
    let secs = Math.max(1, Math.min(120, Number(this.seconds?.value) || 4));
    let played = false;
    if (this.range?.checked && hasCommand('anim.get')) {
      const a = obj(await quiet<unknown>('anim.get', {}));
      const fps = num(a.fps);
      const start = num(a.start);
      const end = num(a.end);
      if (fps && start !== null && end !== null && end > start) {
        secs = Math.min(120, (end - start) / fps);
        if (hasCommand('anim.play')) { await quiet('anim.play', { from: start }); played = true; }
      }
    }
    const { rec, done } = this.recorder();
    rec.start(250);
    const t0 = performance.now();
    while (performance.now() - t0 < secs * 1000 && !this.cancel) {
      this.host.paintNow();
      this.say(`${t('Recording')} ${((performance.now() - t0) / 1000).toFixed(1)} / ${secs.toFixed(1)} s`);
      await wait(1000 / FPS);
    }
    rec.stop();
    if (played && hasCommand('anim.pause')) await quiet('anim.pause', {});
    return done;
  }

  private async turntable(): Promise<Blob | null> {
    const secs = Math.max(1, Math.min(120, Number(this.seconds?.value) || 4));
    const frames = Math.max(2, Math.round(secs * FPS));
    const cam = this.host.camera;
    const yaw = cam.yaw;
    this.host.setAutoFrame(false);
    const { rec, done, push } = this.recorder();
    rec.start(250);
    const t0 = performance.now();
    for (let i = 0; i < frames && !this.cancel; i++) {
      cam.yaw = yaw - (i / frames) * Math.PI * 2;
      this.host.paintNow();
      push();
      this.say(`${t('Recording')} ${i + 1} / ${frames}`);
      const due = t0 + ((i + 1) * 1000) / FPS;
      await wait(Math.max(1, due - performance.now()));
    }
    rec.stop();
    cam.yaw = yaw;
    this.host.redraw();
    return done;
  }
}

