import { onEngineLink } from '../../engine/service';
import type { ShellDeps } from '../../shell/deps';
import { formatKeys } from '../../ui/commands';
import { anyDialogOpen } from '../../ui/dialogs';
import { el, q, setAttr, setClass, setDisabled, setHidden, setStyleVar, setText, setTitle, setValue } from '../../ui/dom';
import { t } from '../../ui/i18n';
import { icon } from '../../ui/icons';
import { textFieldFocused } from '../../ui/native-menu';
import { OFFLINE, engineDown, onEngineDown } from '../offline';
import { MotionPanel } from './motion';
import { AnimStore, type Change } from './store';
import { ROW_H, Timeline } from './timeline';
import { INTERPS, parseValue, showValue, type CacheItem, type Interp, type Row } from './types';
import { label, patchKeyed, repaint, titled } from './widgets';

const OPEN_KEY = 'formfabric.timeline.open';
const KEYED = ['anim.play', 'anim.pause', 'anim.record', 'anim.stop', 'key.set', 'key.remove'];

const INTERP_LABEL: Record<Interp, string> = { step: 'Step', linear: 'Linear', smooth: 'Smooth' };

const HTML =
  '<div class="ff-an-bar">' +
  '<button class="cv-icon-btn" type="button" data-k="motion"></button>' +
  '<i class="ff-an-sep"></i>' +
  '<button class="cv-icon-btn ff-an-rec" type="button" data-k="record"></button>' +
  '<span class="ff-an-job" data-k="job"><span class="ff-an-meter" data-k="meter"><i></i></span><output data-k="pct"></output></span>' +
  '<button class="cv-icon-btn" type="button" data-k="cancel"></button>' +
  '<i class="ff-an-sep"></i>' +
  '<button class="cv-icon-btn" type="button" data-k="back"></button>' +
  '<button class="cv-icon-btn" type="button" data-k="play"></button>' +
  '<button class="cv-icon-btn" type="button" data-k="step"></button>' +
  '<button class="cv-icon-btn" type="button" data-k="loop"></button>' +
  '<i class="ff-an-sep"></i>' +
  '<label class="ff-an-f"><span data-k="l-frame"></span><input class="cv-input" type="number" step="1" min="0" data-k="frame"></label>' +
  '<label class="ff-an-f"><span data-k="l-start"></span><input class="cv-input" type="number" step="1" min="0" data-k="start"></label>' +
  '<label class="ff-an-f"><span data-k="l-end"></span><input class="cv-input" type="number" step="1" min="1" data-k="end"></label>' +
  '<label class="ff-an-f"><span data-k="l-fps"></span><input class="cv-input" type="number" step="1" min="1" max="240" data-k="fps"></label>' +
  '<button class="cv-icon-btn" type="button" data-k="fit"></button>' +
  '<span class="ff-an-gap"></span>' +
  '<span class="ff-an-hint" data-k="hint"></span>' +
  '</div>' +
  '<div class="ff-an-keys">' +
  '<label class="ff-an-f ff-an-grow"><span data-k="l-target"></span><input class="cv-input" type="text" list="ff-an-targets" spellcheck="false" autocomplete="off" data-k="target"></label>' +
  '<datalist id="ff-an-targets" data-k="targets"></datalist>' +
  '<label class="ff-an-f ff-an-grow"><span data-k="l-value"></span><input class="cv-input" type="text" spellcheck="false" autocomplete="off" data-k="value"></label>' +
  '<label class="ff-an-f"><span data-k="l-interp"></span><select class="cv-input" data-k="interp"></select></label>' +
  '<button class="ff-btn" type="button" data-k="addkey"></button>' +
  '<button class="ff-btn" type="button" data-k="rmkey"></button>' +
  '</div>' +
  '<div class="ff-an-layers">' +
  '<button class="cv-icon-btn" type="button" data-k="autokey"></button>' +
  '<i class="ff-an-sep"></i>' +
  '<button class="cv-icon-btn" type="button" data-k="cut"></button>' +
  '<button class="cv-icon-btn" type="button" data-k="merge"></button>' +
  '<button class="cv-icon-btn" type="button" data-k="lock"></button>' +
  '<i class="ff-an-sep"></i>' +
  '<button class="cv-icon-btn" type="button" data-k="earlier"></button>' +
  '<label class="ff-an-f"><span data-k="l-shift"></span><input class="cv-input" type="number" step="1" min="1" max="100000" value="1" data-k="shift"></label>' +
  '<button class="cv-icon-btn" type="button" data-k="later"></button>' +
  '<span class="ff-an-gap"></span>' +
  '<span class="ff-an-hint" data-k="layerhint"></span>' +
  '</div>' +
  '<div class="ff-an-main">' +
  '<div class="ff-an-list"><div class="ff-an-list-head"></div><div class="ff-an-rows" data-k="rows"></div></div>' +
  '<div class="ff-an-cv"><canvas data-k="canvas"></canvas></div>' +
  '</div>' +
  '<div class="ff-an-menu" data-k="menu" hidden>' +
  INTERPS.map((i) => `<button type="button" data-i="${i}"></button>`).join('') +
  '<button type="button" data-i="remove"></button></div>';

export class AnimDock {
  readonly store: AnimStore;
  readonly root: HTMLElement;
  private readonly toggle: HTMLButtonElement;
  private readonly line: Timeline;
  private readonly motion: MotionPanel;
  private readonly tips: Array<{ node: HTMLElement; key: string; cmd: string; need: string[] }> = [];
  private readonly needs: Array<{ node: HTMLButtonElement | HTMLInputElement | HTMLSelectElement; any: string[] }> = [];
  private readonly rows: HTMLElement;
  private readonly menu: HTMLElement;
  private readonly f: Record<string, HTMLInputElement>;
  private readonly b: Record<string, HTMLButtonElement>;
  private readonly playIcon: HTMLElement;
  private readonly pauseIcon: HTMLElement;
  private readonly interp: HTMLSelectElement;
  private readonly playTip: { node: HTMLElement; key: string; cmd: string; need: string[] };
  private isOpen = false;
  private over = false;
  private menuKey: { track: number; frame: number } | null = null;
  private renaming: number | null = null;
  private readonly picked = new Set<number>();

  constructor(private readonly pane: HTMLElement, bar: HTMLElement, private readonly deps: ShellDeps) {
    this.store = new AnimStore(deps.engine, (m) => deps.toast(m, 'error'));
    this.root = el('<section class="ff-an" hidden aria-label="Timeline"></section>');
    this.root.innerHTML = HTML;
    pane.appendChild(this.root);

    this.toggle = el<HTMLButtonElement>(`<button class="cv-icon-btn ff-an-toggle" type="button" aria-expanded="false">${icon('tlTimeline', 18)}</button>`);
    titled(this.toggle, 'Timeline');
    const zoom = q(bar, '[data-k="zoom"]');
    zoom.parentElement?.insertBefore(this.toggle, zoom);

    this.f = {};
    for (const k of ['frame', 'start', 'end', 'fps', 'target', 'value', 'shift']) this.f[k] = q<HTMLInputElement>(this.root, `[data-k="${k}"]`);
    this.b = {};
    for (const k of ['motion', 'record', 'cancel', 'back', 'play', 'step', 'loop', 'fit', 'addkey', 'rmkey', 'autokey', 'cut', 'merge', 'lock', 'earlier', 'later']) this.b[k] = q<HTMLButtonElement>(this.root, `[data-k="${k}"]`);
    this.rows = q(this.root, '[data-k="rows"]');
    this.menu = q(this.root, '[data-k="menu"]');
    this.interp = q<HTMLSelectElement>(this.root, '[data-k="interp"]');

    for (const [k, text] of [['frame', 'Frame'], ['start', 'Start'], ['end', 'End'], ['fps', 'FPS'], ['target', 'Target'], ['value', 'Value'], ['interp', 'Interpolation'], ['shift', 'Frames']]) label(q(this.root, `[data-k="l-${k}"]`), text);
    for (const i of INTERPS) {
      const o = document.createElement('option');
      o.value = i;
      label(o, INTERP_LABEL[i]);
      this.interp.appendChild(o);
    }
    this.interp.value = 'linear';
    for (const i of INTERPS) label(q(this.menu, `[data-i="${i}"]`), INTERP_LABEL[i]);
    label(q(this.menu, '[data-i="remove"]'), 'Remove Key');

    this.b.motion.innerHTML = icon('fileIn', 16);
    this.b.record.innerHTML = icon('tlRecord', 16);
    this.b.cancel.innerHTML = icon('tlStop', 14);
    this.b.back.innerHTML = icon('tlStepBack', 16);
    this.b.play.innerHTML = `<span data-p="play">${icon('simPlay', 16)}</span><span data-p="pause" hidden>${icon('simPause', 16)}</span>`;
    this.playIcon = q(this.b.play, '[data-p="play"]');
    this.pauseIcon = q(this.b.play, '[data-p="pause"]');
    this.b.step.innerHTML = icon('simStep', 16);
    this.b.loop.innerHTML = icon('tlLoop', 16);
    this.b.fit.innerHTML = icon('frameAll', 16);
    this.b.addkey.innerHTML = `${icon('tlKey', 12)}<span></span>`;
    this.b.rmkey.innerHTML = `${icon('delete', 12)}<span></span>`;
    this.b.autokey.innerHTML = icon('tlKey', 16);
    this.b.cut.innerHTML = icon('split', 16);
    this.b.merge.innerHTML = icon('layers', 16);
    this.b.lock.innerHTML = `<span data-p="lock">${icon('lock', 16)}</span><span data-p="unlock" hidden>${icon('unlock', 16)}</span>`;
    this.b.earlier.innerHTML = icon('tlStepBack', 16);
    this.b.later.innerHTML = icon('simStep', 16);
    label(this.b.addkey.children[1] as HTMLElement, 'Add Key');
    label(this.b.rmkey.children[1] as HTMLElement, 'Remove Key');

    this.tip(this.b.motion, 'Import Motion', '', ['motion.import', 'morph.set']);
    this.tip(this.b.record, 'Record', 'anim.record', ['anim.record']);
    this.tip(this.b.cancel, 'Cancel', 'anim.stop', ['anim.stop']);
    this.tip(this.b.back, 'Step back', '', ['anim.get']);
    this.playTip = this.tip(this.b.play, 'Play', 'anim.play', ['cache.frame', 'anim.play']);
    this.tip(this.b.step, 'Step', '', ['anim.get']);
    this.tip(this.b.loop, 'Loop', '', ['anim.get']);
    this.tip(this.b.fit, 'Fit', '', []);
    this.tip(this.b.addkey, 'Add Key', 'key.set', ['key.set']);
    this.tip(this.b.rmkey, 'Remove Key', 'key.remove', ['key.remove']);
    this.tip(this.b.autokey, 'Auto Key', 'anim.auto_key', ['anim.auto_key']);
    this.tip(this.b.cut, 'Cut Layer', 'cache.cut', ['cache.cut']);
    this.tip(this.b.merge, 'Merge Layers', 'cache.merge', ['cache.merge']);
    this.tip(this.b.lock, 'Lock Layer', 'cache.lock', ['cache.lock']);
    this.tip(this.b.earlier, 'Move Earlier', 'cache.move', ['cache.move']);
    this.tip(this.b.later, 'Move Later', 'cache.move', ['cache.move']);
    this.needs.push({ node: this.f.shift, any: ['cache.move'] });
    for (const k of ['frame', 'start', 'end', 'fps']) this.needs.push({ node: this.f[k], any: ['anim.set'] });
    for (const k of ['target', 'value']) this.needs.push({ node: this.f[k], any: ['key.set'] });
    this.needs.push({ node: this.interp, any: ['key.set'] });

    this.motion = new MotionPanel(this.store);
    this.root.appendChild(this.motion.root);

    this.line = new Timeline(q<HTMLCanvasElement>(this.root, '[data-k="canvas"]'), this.store, {
      scrub: (frame) => { if (!this.store.job.running) this.store.setCurrent(frame, true); },
      range: (start, end) => { void this.store.run('anim.set', { start, end }); },
      motion: (id, patch) => { void this.store.run('motion.set', { id, ...patch }); },
      moveKey: (track, from, to) => { void this.moveKey(track, from, to); },
      keyMenu: (track, frame, x, y) => this.showMenu(track, frame, x, y),
      pickRow: (row) => { this.pickRow(row); },
      pickKey: (track, frame) => { this.pickKey(track, frame); },
      rowScroll: (px) => { this.rows.scrollTop = px; },
    });

    this.wire();
    this.store.onChange((c) => this.changed(c));
    repaint(() => { this.paintTips(); this.paintAria(); });
    this.paintAll();
    onEngineUp(() => { void this.store.init(); });
    onEngineDown(() => this.paintAvail());
    void this.store.init();
    if (localStorage.getItem(OPEN_KEY) === '1') this.setOpen(true);
  }

  get open(): boolean {
    return this.isOpen;
  }

  setOpen(on: boolean): void {
    if (on === this.isOpen) return;
    this.isOpen = on;
    localStorage.setItem(OPEN_KEY, on ? '1' : '0');
    setClass(this.toggle, 'is-active', on);
    this.toggle.setAttribute('aria-expanded', String(on));
    setHidden(this.root, !on);
    setClass(this.pane, 'has-timeline', on);
    if (on) { void this.store.refresh(); this.paintAll(); this.line.invalidate(); }
    else { this.store.pause(); this.motion.setOpen(false); this.hideMenu(); }
  }

  private paintAria(): void {
    const s = t('Timeline');
    if (this.root.getAttribute('aria-label') !== s) this.root.setAttribute('aria-label', s);
  }

  private tip(node: HTMLElement, key: string, cmd: string, need: string[]): { node: HTMLElement; key: string; cmd: string; need: string[] } {
    const entry = { node, key, cmd, need };
    this.tips.push(entry);
    node.addEventListener('pointerenter', () => this.paintTip(entry));
    if (node instanceof HTMLButtonElement && need.length) this.needs.push({ node, any: need });
    return entry;
  }

  private missing(need: string[]): boolean {
    return need.length > 0 && !need.some((c) => this.store.has(c));
  }

  private paintTip(x: { node: HTMLElement; key: string; cmd: string; need: string[] }): void {
    if (this.missing(x.need)) { setTitle(x.node, t('Unavailable in this engine build')); return; }
    const c = x.cmd ? this.deps.commands.get(x.cmd) : undefined;
    const k = c?.keys?.[0] ? ` (${formatKeys(c.keys[0])})` : '';
    const text = t(x.key) + k;
    setTitle(x.node, text);
    setAttr(x.node, 'aria-label', text);
  }

  private paintTips(): void {
    for (const x of this.tips) this.paintTip(x);
  }

  private wire(): void {
    this.toggle.addEventListener('click', (e) => { this.setOpen(!this.isOpen); if (e.detail > 0) this.toggle.blur(); });
    this.b.motion.addEventListener('click', () => { this.motion.setOpen(!this.motion.open); this.motion.sync(); });
    this.b.record.addEventListener('click', () => { this.record(); });
    this.b.cancel.addEventListener('click', () => { void this.store.cancel(); });
    this.b.play.addEventListener('click', () => { this.togglePlay(); });
    this.b.back.addEventListener('click', () => { this.stepBy(-1); });
    this.b.step.addEventListener('click', () => { this.stepBy(1); });
    this.b.loop.addEventListener('click', () => { this.store.loop = !this.store.loop; setClass(this.b.loop, 'is-on', this.store.loop); });
    this.b.fit.addEventListener('click', () => { this.line.fit(); });
    this.b.addkey.addEventListener('click', () => { void this.addKey(); });
    this.b.rmkey.addEventListener('click', () => { void this.removeKey(); });
    this.b.autokey.addEventListener('click', () => { void this.store.run('anim.auto_key', { on: !this.store.anim.autoKey }); });
    this.b.cut.addEventListener('click', () => { const c = this.selCache(); if (c) void this.store.run('cache.cut', { id: c.id, frame: this.store.current }); });
    this.b.merge.addEventListener('click', () => { if (this.picked.size > 1) void this.store.run('cache.merge', { ids: Array.from(this.picked) }); });
    this.b.lock.addEventListener('click', () => { const c = this.selCache(); if (c) void this.store.run('cache.lock', { id: c.id, locked: !c.locked }); });
    this.b.earlier.addEventListener('click', () => { this.shiftBy(-1); });
    this.b.later.addEventListener('click', () => { this.shiftBy(1); });
    for (const k of Object.keys(this.b)) this.b[k].addEventListener('click', (e) => { if (e.detail > 0) this.b[k].blur(); });
    setClass(this.b.loop, 'is-on', this.store.loop);

    const num = (k: string): number | null => { const v = Number(this.f[k].value); return this.f[k].value.trim() !== '' && Number.isFinite(v) ? Math.round(v) : null; };
    this.f.frame.addEventListener('change', () => { const v = num('frame'); if (v !== null) this.store.setCurrent(Math.max(0, v), true); });
    this.f.start.addEventListener('change', () => {
      const v = num('start');
      const a = this.store.anim;
      if (v === null || v < 0 || v >= a.end) { this.paintFields(); return; }
      void this.store.run('anim.set', { start: v });
    });
    this.f.end.addEventListener('change', () => {
      const v = num('end');
      const a = this.store.anim;
      if (v === null || v <= a.start) { this.paintFields(); return; }
      void this.store.run('anim.set', { end: v });
    });
    this.f.fps.addEventListener('change', () => {
      const v = num('fps');
      if (v === null || v < 1 || v > 240) { this.paintFields(); return; }
      void this.store.run('anim.set', { fps: v });
    });
    for (const k of ['target', 'value']) this.f[k].addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); void this.addKey(); } });

    this.menu.addEventListener('click', (e) => {
      const b = e.target instanceof Element ? e.target.closest<HTMLElement>('[data-i]') : null;
      if (!b || !this.menuKey) return;
      const { track, frame } = this.menuKey;
      this.hideMenu();
      if (b.dataset.i === 'remove') void this.removeKey(track, frame);
      else void this.setInterp(track, frame, b.dataset.i as Interp);
    });
    document.addEventListener('pointerdown', (e) => {
      if (!this.menu.hidden && !(e.target instanceof Node && this.menu.contains(e.target))) this.hideMenu();
    }, true);
    this.pane.addEventListener('pointerenter', () => { this.over = true; });
    this.pane.addEventListener('pointerleave', () => { this.over = false; });
    window.addEventListener('keydown', (e) => this.keydown(e), true);
  }

  private keydown(e: KeyboardEvent): void {
    if (e.defaultPrevented || !this.isOpen || anyDialogOpen()) return;
    if (!this.over && !this.root.contains(document.activeElement) && this.deps.layout.get() !== '3d') return;
    if (e.key === 'Escape' && !this.menu.hidden) { this.hideMenu(); e.preventDefault(); return; }
    const cmd = this.deps.commands.forEvent(e);
    if (!cmd || !KEYED.includes(cmd.id) || !this.store.has(cmd.id)) return;
    if (textFieldFocused() && !(e.ctrlKey || e.metaKey)) return;
    e.preventDefault();
    e.stopPropagation();
    switch (cmd.id) {
      case 'anim.play': this.togglePlay(); break;
      case 'anim.pause': this.store.pause(); break;
      case 'anim.record': if (this.store.job.running) void this.store.cancel(); else this.record(); break;
      case 'anim.stop': if (this.store.job.running) void this.store.cancel(); else this.store.pause(); break;
      case 'key.set': void this.addKey(); break;
      case 'key.remove': void this.removeKey(); break;
      default: break;
    }
  }

  private togglePlay(): void {
    if (this.store.job.running) return;
    if (this.store.playing) this.store.pause(); else this.store.play();
  }

  private stepBy(d: number): void {
    this.store.pause();
    const a = this.store.anim;
    const next = Math.max(0, Math.min(Math.max(a.end, a.start + 1), this.store.current + d));
    this.store.setCurrent(next, true);
  }

  private record(): void {
    const a = this.store.anim;
    const s = Math.round(Number(this.f.start.value));
    const e = Math.round(Number(this.f.end.value));
    const ok = Number.isFinite(s) && Number.isFinite(e) && s >= 0 && e > s;
    void this.store.record(ok ? s : a.start, ok ? e : a.end, `Rec ${a.caches.length + 1}`);
  }

  private selCache(): CacheItem | undefined {
    const row = this.line.selRow;
    return row && row.kind === 'cache' ? this.store.anim.caches.find((c) => c.id === row.id) : undefined;
  }

  private shiftBy(dir: number): void {
    const c = this.selCache();
    const n = Math.round(Number(this.f.shift.value));
    if (!c || !Number.isFinite(n) || n < 1) return;
    void this.store.run('cache.move', { id: c.id, frames: dir * n });
  }

  private paintLayers(): void {
    const s = this.store;
    const a = s.anim;
    const c = this.selCache();
    const run = s.job.running;
    setClass(this.b.autokey, 'is-on', a.autoKey);
    setAttr(this.b.autokey, 'aria-pressed', String(a.autoKey));
    const free = !!c && !c.locked && !run;
    const inside = !!c && s.current > c.start && s.current < c.start + c.frames;
    if (!this.missing(['cache.cut'])) setDisabled(this.b.cut, !free || !inside);
    if (!this.missing(['cache.merge'])) setDisabled(this.b.merge, run || this.picked.size < 2 || Array.from(this.picked).some((id) => a.caches.find((k) => k.id === id)?.locked));
    if (!this.missing(['cache.lock'])) setDisabled(this.b.lock, !c || run);
    for (const k of ['earlier', 'later']) if (!this.missing(['cache.move'])) setDisabled(this.b[k], !free);
    setDisabled(this.f.shift, this.missing(['cache.move']));
    setClass(this.b.lock, 'is-on', !!c && c.locked);
    setHidden(q(this.b.lock, '[data-p="lock"]'), !(c && c.locked));
    setHidden(q(this.b.lock, '[data-p="unlock"]'), !!(c && c.locked));
    const hint = !c ? '' : c.locked ? t('Locked') : c.offset !== 0 ? `${t('Offset')} ${c.offset > 0 ? '+' : ''}${c.offset}` : '';
    setText(q(this.root, '[data-k="layerhint"]'), hint);
  }

  private trackOf(id: number | null): { id: number; target: string } | undefined {
    return id === null ? undefined : this.store.anim.tracks.find((x) => x.id === id);
  }

  private async addKey(): Promise<void> {
    const a = this.store.anim;
    const target = this.f.target.value.trim();
    if (!target) { this.f.target.focus(); return; }
    let value = parseValue(this.f.value.value);
    const tr = a.tracks.find((x) => x.target === target);
    if (value === undefined && tr && tr.keys.length) {
      const near = tr.keys.reduce((p, k) => (Math.abs(k.frame - this.store.current) < Math.abs(p.frame - this.store.current) ? k : p));
      value = near.value;
    }
    if (value === undefined) { this.f.value.focus(); return; }
    const r = await this.store.run('key.set', { target, frame: this.store.current, value, interp: this.interp.value });
    if (r !== undefined) { this.line.selFrame = this.store.current; await this.store.refresh(); this.line.selTrack = this.store.anim.tracks.find((x) => x.target === target)?.id ?? null; this.paintKeys(); }
  }

  private async removeKey(track: number | null = this.line.selTrack, frame = this.line.selFrame): Promise<void> {
    const tr = this.trackOf(track);
    if (!tr || frame < 0) return;
    await this.store.run('key.remove', { target: tr.target, frame });
    this.line.selFrame = -1;
    await this.store.refresh();
    this.paintKeys();
  }

  private async setInterp(track: number, frame: number, interp: Interp): Promise<void> {
    const tr = this.store.anim.tracks.find((x) => x.id === track);
    const k = tr?.keys.find((x) => x.frame === frame);
    if (!tr || !k) return;
    await this.store.run('key.set', { target: tr.target, frame, value: k.value, interp });
    await this.store.refresh();
  }

  private async moveKey(track: number, from: number, to: number): Promise<void> {
    const tr = this.store.anim.tracks.find((x) => x.id === track);
    const k = tr?.keys.find((x) => x.frame === from);
    if (!tr || !k) return;
    const ok = await this.store.run('key.set', { target: tr.target, frame: to, value: k.value, interp: k.interp });
    if (ok === undefined) return;
    await this.store.run('key.remove', { target: tr.target, frame: from });
    await this.store.refresh();
  }

  private showMenu(track: number, frame: number, x: number, y: number): void {
    this.menuKey = { track, frame };
    const r = this.root.getBoundingClientRect();
    const left = Math.max(0, Math.min(r.width - 120, x - r.left));
    const top = Math.max(0, Math.min(r.height - 110, y - r.top));
    setStyleVar(this.menu, '--x', `${left}px`);
    setStyleVar(this.menu, '--y', `${top}px`);
    const tr = this.store.anim.tracks.find((v) => v.id === track);
    const cur = tr?.keys.find((v) => v.frame === frame)?.interp;
    for (const i of INTERPS) setClass(q(this.menu, `[data-i="${i}"]`), 'is-on', i === cur);
    setHidden(this.menu, false);
  }

  private hideMenu(): void {
    this.menuKey = null;
    setHidden(this.menu, true);
  }

  private pickRow(row: Row | null, multi = false): void {
    this.line.selRow = row;
    if (row?.kind === 'cache') {
      if (multi && this.picked.has(row.id)) this.picked.delete(row.id);
      else {
        if (!multi) this.picked.clear();
        this.picked.add(row.id);
      }
    } else if (!multi) this.picked.clear();
    if (row?.kind === 'cache') {
      this.store.activeId = row.id;
      if ('active' in this.store.props('cache.set')) void this.store.run('cache.set', { id: row.id, active: true }, true);
    }
    if (row?.kind === 'track') {
      const tr = this.trackOf(row.id);
      if (tr) setValue(this.f.target, tr.target);
    }
    this.paintList();
    this.paintLayers();
    this.line.invalidate();
  }

  private pickKey(track: number | null, frame: number): void {
    this.line.selTrack = track;
    this.line.selFrame = frame;
    const tr = this.trackOf(track);
    const k = this.store.anim.tracks.find((x) => x.id === track)?.keys.find((x) => x.frame === frame);
    if (tr && k) {
      setValue(this.f.target, tr.target);
      setValue(this.f.value, showValue(k.value));
      if (this.interp.value !== k.interp) this.interp.value = k.interp;
    }
    this.paintKeys();
  }

  private changed(c: Change): void {
    if (c === 'frame') { this.paintFrame(); return; }
    if (c === 'job') { this.paintJob(); this.paintFields(); this.paintLayers(); return; }
    this.paintAll();
  }

  private paintAll(): void {
    this.paintTips();
    this.paintAvail();
    this.paintFields();
    this.paintFrame();
    this.paintList();
    this.paintJob();
    this.paintKeys();
    this.paintLayers();
    this.motion.sync();
    this.line.invalidate();
  }

  private paintAvail(): void {
    const s = this.store;
    const layered: HTMLElement[] = [this.b.cut, this.b.merge, this.b.lock, this.b.earlier, this.b.later];
    for (const n of this.needs) {
      if (n.node === this.b.cancel || (layered.includes(n.node) && !this.missing(n.any))) continue;
      const off = this.missing(n.any) || (s.job.running && n.node !== this.b.cancel && (n.node instanceof HTMLButtonElement || n.node === this.f.frame) && n.node !== this.b.fit);
      setDisabled(n.node, off);
    }
    setDisabled(this.b.cancel, !s.job.running || !s.has('anim.stop'));
    setText(q(this.root, '[data-k="hint"]'), engineDown() && !s.has('anim.get') ? t(OFFLINE) : s.loaded && !s.has('anim.get') ? t('Unavailable in this engine build') : '');
  }

  private paintFields(): void {
    const a = this.store.anim;
    setValue(this.f.start, String(a.start));
    setValue(this.f.end, String(a.end));
    setValue(this.f.fps, String(Math.round(a.fps * 100) / 100));
    this.paintFrame();
  }

  private paintFrame(): void {
    const s = this.store;
    setValue(this.f.frame, String(s.current));
    setHidden(this.playIcon, s.playing);
    setHidden(this.pauseIcon, !s.playing);
    setClass(this.b.play, 'is-on', s.playing);
    this.paintLayers();
    const key = s.playing ? 'Pause' : 'Play';
    if (this.playTip.key !== key) { this.playTip.key = key; this.paintTip(this.playTip); }
    this.line.invalidate();
  }

  private paintJob(): void {
    const j = this.store.job;
    setClass(q(this.root, '[data-k="job"]'), 'is-on', j.running);
    setStyleVar(q(this.root, '[data-k="meter"]'), '--p', String(j.progress));
    setText(q(this.root, '[data-k="pct"]'), j.running ? `${Math.round(j.progress * 100)}%` : '');
    setClass(this.b.record, 'is-on', j.running);
    this.paintAvail();
  }

  private paintKeys(): void {
    const has = this.line.selTrack !== null && this.line.selFrame >= 0;
    if (!this.missing(['key.remove'])) setDisabled(this.b.rmkey, !has || this.store.job.running);
    const tr = this.store.anim.tracks;
    patchKeyed(q(this.root, '[data-k="targets"]'), tr, {
      key: (x) => x.target,
      create: () => document.createElement('option'),
      update: (n, x) => { const o = n as HTMLOptionElement; if (o.value !== x.target) o.value = x.target; },
    });
    this.line.invalidate();
  }

  private paintList(): void {
    const rows = this.line.rows();
    const a = this.store.anim;
    for (const id of Array.from(this.picked)) if (!a.caches.some((k) => k.id === id)) this.picked.delete(id);
    patchKeyed(this.rows, rows, {
      key: (r) => `${r.kind}:${r.id}`,
      create: (r) => this.makeRow(r),
      update: (n, r) => this.updateRow(n, r),
    });
    this.line.setRowY(this.line.rowY);
    this.rows.scrollTop = this.line.rowY;
  }

  private makeRow(r: Row): HTMLElement {
    const n = el(
      `<div class="ff-an-item" data-kind="${r.kind}" style="height:${ROW_H}px">` +
      '<button class="cv-icon-btn ff-an-vis" type="button" data-a="vis"><span data-v="on"></span><span data-v="off" hidden></span></button>' +
      '<span class="ff-an-ico"></span><span class="ff-an-name"></span><input class="cv-input ff-an-ren" type="text" hidden><span class="ff-an-lk"></span>' +
      '<button class="cv-icon-btn ff-an-del" type="button" data-a="del"></button></div>');
    q(n, '[data-v="on"]').innerHTML = icon('eye', 14);
    q(n, '[data-v="off"]').innerHTML = icon('eyeOff', 14);
    const pinned = r.kind === 'track' && (this.store.anim.tracks.find((k) => k.id === r.id)?.target ?? '').startsWith('pin:');
    q(n, '.ff-an-ico').innerHTML = icon(pinned ? 'pinTool' : r.kind === 'track' ? 'tlKey' : r.kind === 'motion' ? 'pose' : 'layers', 12);
    q(n, '.ff-an-lk').innerHTML = icon('lock', 11);
    q(n, '.ff-an-del').innerHTML = icon('delete', 14);
    const vis = q<HTMLElement>(n, '[data-a="vis"]');
    const del = q<HTMLElement>(n, '[data-a="del"]');
    titled(del, 'Delete');
    if (r.kind !== 'cache') setHidden(vis, true);
    if (r.kind === 'track') setHidden(del, true);
    const id = (): number => Number(n.dataset.id);
    n.dataset.id = String(r.id);
    n.addEventListener('click', (e) => {
      if (e.target instanceof Element && e.target.closest('button,input')) return;
      this.pickRow({ kind: r.kind, id: id() }, e.shiftKey || e.ctrlKey || e.metaKey);
    });
    vis.addEventListener('click', () => {
      const c = this.store.anim.caches.find((k) => k.id === id());
      if (c) void this.store.run('cache.set', { id: c.id, visible: !c.visible });
      vis.blur();
    });
    del.addEventListener('click', () => {
      void this.store.run(r.kind === 'cache' ? 'cache.remove' : 'motion.remove', { id: id() });
      del.blur();
    });
    const ren = q<HTMLInputElement>(n, '.ff-an-ren');
    const name = q(n, '.ff-an-name');
    if (r.kind === 'cache') {
      name.addEventListener('dblclick', () => {
        const c = this.store.anim.caches.find((k) => k.id === id());
        if (!c) return;
        this.renaming = c.id;
        ren.value = c.name;
        setHidden(ren, false);
        setHidden(name, true);
        ren.focus();
        ren.select();
      });
      const done = (commit: boolean): void => {
        if (this.renaming === null) return;
        const cid = this.renaming;
        this.renaming = null;
        setHidden(ren, true);
        setHidden(name, false);
        const v = ren.value.trim();
        const c = this.store.anim.caches.find((k) => k.id === cid);
        if (commit && v && c && v !== c.name) void this.store.run('cache.set', { id: cid, name: v });
      };
      ren.addEventListener('keydown', (e) => {
        e.stopPropagation();
        if (e.key === 'Enter') { e.preventDefault(); done(true); }
        if (e.key === 'Escape') { e.preventDefault(); done(false); }
      });
      ren.addEventListener('blur', () => done(true));
    }
    return n;
  }

  private updateRow(n: HTMLElement, r: Row): void {
    const a = this.store.anim;
    let text = '';
    let hiddenRow = false;
    let active = false;
    if (r.kind === 'cache') {
      const c = a.caches.find((k) => k.id === r.id);
      text = c?.name ?? '';
      setClass(q(n, '.ff-an-lk'), 'is-on', !!c?.locked);
      setClass(n, 'is-trim', c?.kind === 'trim');
      hiddenRow = c ? !c.visible : false;
      active = r.id === this.store.activeId;
      setHidden(q(n, '[data-v="on"]'), hiddenRow);
      setHidden(q(n, '[data-v="off"]'), !hiddenRow);
    } else if (r.kind === 'motion') {
      text = a.motions.find((k) => k.id === r.id)?.name ?? '';
    } else {
      text = a.tracks.find((k) => k.id === r.id)?.target ?? '';
    }
    setText(q(n, '.ff-an-name'), text);
    setTitle(q(n, '.ff-an-name'), text);
    setClass(n, 'is-hidden', hiddenRow);
    setClass(n, 'is-active', active);
    const sel = this.line.selRow;
    setClass(n, 'is-sel', (!!sel && sel.kind === r.kind && sel.id === r.id) || (r.kind === 'cache' && this.picked.has(r.id)));
  }
}

function onEngineUp(cb: () => void): void {
  onEngineLink((s) => { if (s === 'up') cb(); });
}
