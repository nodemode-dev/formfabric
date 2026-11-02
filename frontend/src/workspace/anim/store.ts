import type { EngineApi } from '../../shell/deps';
import { parseAnim, parseJob, type AnimState, type CacheItem, type JobInfo } from './types';
import { EMPTY } from './types';

export type Change = 'data' | 'frame' | 'job' | 'avail';

const WATCH = /^(anim|motion|key|cache|morph|file|history|body)\./;

export class AnimStore {
  anim: AnimState = EMPTY;
  job: JobInfo = { running: false, progress: 0, text: '', failed: '', playing: false, playFrame: null };
  commands = new Set<string>();
  schemas = new Map<string, Record<string, unknown>>();
  current = 0;
  playing = false;
  loop = true;
  loaded = false;
  interacting = false;
  activeId: number | null = null;
  private readonly listeners = new Set<(c: Change) => void>();
  private inflight: Promise<void> | null = null;
  private again = false;
  private timer = 0;
  private wantFrame = -1;
  private sentFrame = -1;
  private pumping = false;
  private raf = 0;
  private t0 = 0;
  private f0 = 0;
  private streamBusy = false;
  private streamWant = -1;
  private serverPlay = false;
  private poll = 0;
  private jobTimer = 0;
  private seenRunning = false;
  private finishing = false;
  private jobStart = 0;

  constructor(private readonly engine: EngineApi, private readonly fail: (message: string) => void) {
    engine.onEngineChanged((e) => {
      if (WATCH.test(e.name)) this.later();
    });
  }

  onChange(cb: (c: Change) => void): () => void {
    this.listeners.add(cb);
    return () => { this.listeners.delete(cb); };
  }

  emit(c: Change): void {
    for (const cb of this.listeners) cb(c);
  }

  has(command: string): boolean {
    return this.commands.has(command);
  }

  props(command: string): Record<string, Record<string, unknown>> {
    const p = this.schemas.get(command)?.properties;
    return p && typeof p === 'object' ? (p as Record<string, Record<string, unknown>>) : {};
  }

  async init(): Promise<void> {
    try {
      const cat = await this.engine.catalogue();
      this.commands = new Set(cat.commands.map((c) => c.id));
      this.schemas = new Map(cat.commands.map((c) => [c.id, c.inputSchema as unknown as Record<string, unknown>]));
    } catch { this.commands = new Set(); }
    this.loaded = true;
    this.emit('avail');
    if (this.has('anim.get')) await this.refresh();
  }

  refresh(): Promise<void> {
    if (!this.has('anim.get')) return Promise.resolve();
    if (this.inflight) { this.again = true; return this.inflight; }
    this.inflight = this.pull().finally(() => { this.inflight = null; });
    return this.inflight;
  }

  private async pull(): Promise<void> {
    do {
      this.again = false;
      try {
        const next = parseAnim(await this.engine.call<unknown>('anim.get', {}));
        this.anim = next;
        if (!this.interacting && !this.playing) this.current = next.current;
        if (this.activeId === null || !next.caches.some((c) => c.id === this.activeId)) this.activeId = next.active ?? next.caches[0]?.id ?? null;
        else if (next.active !== null && next.active !== this.activeId && !this.interacting) this.activeId = next.active;
        this.emit('data');
      } catch { /* engine away */ }
    } while (this.again);
  }

  async run<T>(name: string, args: Record<string, unknown> = {}, quiet = false): Promise<T | undefined> {
    if (!this.has(name)) return undefined;
    try {
      const r = await this.engine.call<T>(name, args);
      if (name !== 'anim.get' && name !== 'job.state' && name !== 'cache.frame') this.later();
      return r;
    } catch (err) {
      if (!quiet) this.fail(err instanceof Error ? err.message : String(err));
      return undefined;
    }
  }

  private later(): void {
    window.clearTimeout(this.timer);
    this.timer = window.setTimeout(() => { void this.refresh(); }, 120);
  }

  async read<T>(name: string, args: Record<string, unknown> = {}): Promise<T | undefined> {
    return this.run<T>(name, args, true);
  }

  cacheAt(frame: number): CacheItem | undefined {
    const covers = (c: CacheItem): boolean => c.kind === 'pattern' && c.visible && c.frames > 0 && frame >= c.start && frame < c.start + c.frames;
    for (let i = this.anim.caches.length - 1; i >= 0; i -= 1) if (covers(this.anim.caches[i])) return this.anim.caches[i];
    return undefined;
  }

  setCurrent(frame: number, persist: boolean): void {
    const f = Math.max(0, Math.round(frame));
    if (f === this.current && !persist) return;
    this.current = f;
    this.emit('frame');
    this.stream(f);
    if (persist) {
      this.wantFrame = f;
      void this.pump();
    }
  }

  private async pump(): Promise<void> {
    if (this.pumping) return;
    this.pumping = true;
    try {
      while (this.wantFrame !== this.sentFrame) {
        const f = this.wantFrame;
        this.sentFrame = f;
        await this.run('anim.set', { current: f }, true);
      }
    } finally { this.pumping = false; }
  }

  private stream(frame: number): void {
    if (!this.has('cache.frame')) return;
    const c = this.cacheAt(frame);
    if (!c) return;
    this.streamWant = frame;
    if (this.streamBusy) return;
    this.streamBusy = true;
    void (async () => {
      try {
        while (this.streamWant >= 0) {
          const f = this.streamWant;
          this.streamWant = -1;
          const cache = this.cacheAt(f);
          if (cache) await this.run('cache.frame', { frame: f }, true);
        }
      } finally { this.streamBusy = false; }
    })();
  }

  canPlay(): boolean {
    return this.has('cache.frame') || this.has('anim.play');
  }

  play(): void {
    if (this.playing || !this.canPlay()) return;
    const a = this.anim;
    const from = this.current >= a.end || this.current < a.start ? a.start : this.current;
    this.current = from;
    this.playing = true;
    const cache = this.cacheAt(from) ?? a.caches.find((c) => c.kind === 'pattern' && c.visible && c.frames > 0);
    if (cache && this.has('anim.play')) {
      const inside = from >= cache.start && from < cache.start + cache.frames;
      void this.serverStart(inside ? from : cache.start);
    } else {
      this.clientStart(from);
    }
    this.emit('frame');
  }

  private clientStart(from: number): void {
    this.t0 = performance.now();
    this.f0 = from;
    this.raf = requestAnimationFrame(this.tick);
  }

  private async serverStart(from: number): Promise<void> {
    const r = await this.run('anim.play', { from, loop: this.loop }, true);
    if (!this.playing) {
      if (r !== undefined) void this.run('anim.pause', {}, true);
      return;
    }
    if (r === undefined) { this.clientStart(from); return; }
    this.serverPlay = true;
    window.clearInterval(this.poll);
    this.poll = window.setInterval(() => { void this.pollPlay(); }, 80);
  }

  private async pollPlay(): Promise<void> {
    const j = parseJob(await this.read<unknown>('job.state', {}));
    if (!this.playing || !this.serverPlay) return;
    if (j.playFrame !== null) this.current = j.playFrame;
    if (!j.playing) {
      this.playing = false;
      this.serverPlay = false;
      window.clearInterval(this.poll);
      this.wantFrame = this.current;
      void this.pump();
    }
    this.emit('frame');
  }

  private readonly tick = (now: number): void => {
    if (!this.playing) return;
    const a = this.anim;
    const span = Math.max(1, a.end - a.start + 1);
    const steps = Math.floor(((now - this.t0) / 1000) * Math.max(1, a.fps));
    let f = this.f0 + steps;
    if (f > a.end) {
      if (this.loop) {
        f = a.start + ((f - a.start) % span);
      } else {
        this.setCurrent(a.end, false);
        this.pause();
        return;
      }
    }
    if (f !== this.current) this.setCurrent(f, false);
    this.raf = requestAnimationFrame(this.tick);
  };

  pause(): void {
    if (!this.playing) return;
    this.playing = false;
    cancelAnimationFrame(this.raf);
    window.clearInterval(this.poll);
    if (this.serverPlay) void this.run('anim.pause', {}, true);
    this.serverPlay = false;
    this.wantFrame = this.current;
    void this.pump();
    this.emit('frame');
  }

  async record(start: number, end: number, name: string): Promise<void> {
    if (!this.has('anim.record') || this.job.running) return;
    this.pause();
    this.job = { running: true, progress: 0, text: '', failed: '', playing: false, playFrame: null };
    this.seenRunning = false;
    this.jobStart = performance.now();
    this.emit('job');
    this.jobTimer = window.setInterval(() => { void this.pollJob(); }, 250);
    const r = await this.run('anim.record', { start, end, name });
    if (r === undefined) { this.finishJob(); return; }
    await this.pollJob();
  }

  private async pollJob(): Promise<void> {
    if (!this.has('job.state')) { this.finishJob(); return; }
    const raw = await this.read<unknown>('job.state', {});
    if (raw === undefined) return;
    const next = parseJob(raw);
    if (next.failed) this.fail(next.failed);
    if (next.running) this.seenRunning = true;
    if (!next.running) {
      if (this.seenRunning || next.text === 'done' || performance.now() - this.jobStart > 1500 || next.failed) this.finishJob();
      return;
    }
    this.job = next;
    this.emit('job');
  }

  private finishJob(): void {
    if (this.finishing) return;
    this.finishing = true;
    window.clearInterval(this.jobTimer);
    void (async () => {
      if (this.has('job.harvest')) await this.run('job.harvest', {}, true);
      this.job = { running: false, progress: 0, text: '', failed: '', playing: false, playFrame: null };
      this.emit('job');
      this.finishing = false;
      await this.refresh();
    })();
  }

  async cancel(): Promise<void> {
    await this.run('anim.stop', {}, true);
    await this.pollJob();
  }
}
