import { fromHtml as el, q, setClass, setHidden, setText } from './dom';
import type { DockDeps, EngineEvent, Link } from './link';
import { MarkdownStream } from './markdown';

interface Part { kind: 'text' | 'tool'; text?: string; id?: string; name?: string; input?: string; status?: string; images?: string[]; objects?: number[] }
interface Meta { seconds?: number; turns?: number | null; costUsd?: number | null; inputTokens?: number; outputTokens?: number }
interface Message { role: 'user' | 'assistant'; text?: string; note?: string; parts?: Part[]; error?: string; meta?: Meta }
interface Usage { input: number; output: number; costUsd: number; context: number; window: number; model: string }
interface State { busy: boolean; messages: Message[]; turn: number; session: string; usage?: Usage }
interface Session { id: string; title: string; modified: number; current: boolean }

function tokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(n % 1_000_000 ? 1 : 0)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(n >= 100_000 ? 0 : 1)}k`;
  return String(n);
}

export class ChatPane {
  readonly root: HTMLElement;
  onBusy: (busy: boolean) => void = () => undefined;
  onFail: (err: unknown) => void = () => undefined;
  onHandoff: (session: string) => void = () => undefined;
  onObjects: (ids: number[]) => void = () => undefined;
  private readonly log: HTMLElement;
  private readonly empty: HTMLElement;
  private readonly meter: HTMLElement;
  private readonly pick: HTMLElement;
  private readonly pickSelect: HTMLSelectElement;
  private reply: HTMLElement | null = null;
  private md: MarkdownStream | null = null;
  private usage: Usage = { input: 0, output: 0, costUsd: 0, context: 0, window: 0, model: '' };
  private readonly chips = new Map<string, HTMLElement>();
  private readonly off: Array<() => void> = [];
  private busy = false;
  private loaded = false;

  constructor(private readonly link: Link, private readonly deps: DockDeps) {
    const t = deps.t;
    this.root = el(`<div class="ff-chat" hidden>
<div class="ff-chat-bar"><span class="ff-chat-status ff-note"></span><span class="ff-chat-usage" title="${t('Memory: how much of Claude\'s working memory this conversation uses (it reads all of it again with each message). Reply: tokens written so far. Chat: everything sent and written in this conversation, and what it has cost.')}"></span><span class="ff-dock-spacer"></span>
<button type="button" class="ff-btn ff-chat-stop" hidden>${t('Stop')}</button>
<button type="button" class="ff-btn ff-chat-adopt" title="${t('Bring a conversation you had in a Claude Code tab into this chat')}">${t('From Claude Code...')}</button>
<button type="button" class="ff-btn ff-chat-handoff" title="${t('Continue this conversation in a Claude Code terminal tab')}">${t('Open in Claude Code')}</button>
<button type="button" class="ff-btn ff-chat-new">${t('New Chat')}</button></div>
<div class="ff-chat-pick" hidden><span class="ff-note">${t('Conversation')}</span><select class="ff-input ff-chat-pick-select" data-no-i18n aria-label="${t('Conversation')}"></select><button type="button" class="ff-btn ff-chat-pick-ok">${t('Continue Here')}</button><button type="button" class="ff-btn ff-chat-pick-x">${t('Cancel')}</button></div>
<div class="ff-chat-log" role="log" aria-live="polite"><p class="ff-chat-empty ff-note">${t('Ask Claude about your garment in the line below: draw panels, shape curves, add inner lines or check measurements. It works through formfabric\'s tools, every change shows here and in the views, and each one can be undone.')}</p></div>
</div>`);
    this.log = q(this.root, '.ff-chat-log');
    this.empty = q(this.root, '.ff-chat-empty');
    this.meter = q(this.root, '.ff-chat-usage');
    this.pick = q(this.root, '.ff-chat-pick');
    this.pickSelect = q<HTMLSelectElement>(this.root, '.ff-chat-pick-select');
    q(this.root, '.ff-chat-stop').addEventListener('click', () => void this.link.cmd('chat.cancel').catch((e) => this.onFail(e)));
    q(this.root, '.ff-chat-new').addEventListener('click', () => void this.link.cmd('chat.reset').catch((e) => this.onFail(e)));
    q(this.root, '.ff-chat-handoff').addEventListener('click', () => void this.handoff());
    q(this.root, '.ff-chat-adopt').addEventListener('click', () => void this.adopt());
    q(this.root, '.ff-chat-pick-x').addEventListener('click', () => setHidden(this.pick, true));
    q(this.root, '.ff-chat-pick-ok').addEventListener('click', () => void this.adoptChosen());
    const on = (type: string, fn: (e: EngineEvent) => void): void => { this.off.push(link.on(type, fn)); };
    on('chat.usage', (e) => this.showUsage({ ...this.usage, context: Number(e.context ?? 0), window: Number(e.window ?? 0) || this.usage.window, model: String(e.model ?? this.usage.model) }, Number(e.output ?? 0)));
    on('chat.adopted', () => { this.loaded = false; this.log.replaceChildren(this.empty); this.reply = null; this.md = null; this.chips.clear(); void this.load(); });
    on('chat.start', (e) => { this.user(String(e.text ?? ''), String(e.note ?? '')); this.begin(); });
    on('chat.delta', (e) => this.delta(String(e.text ?? '')));
    on('chat.tool', (e) => this.tool({ kind: 'tool', id: String(e.id ?? ''), name: String(e.name ?? ''), input: String(e.input ?? ''), status: 'running', objects: Array.isArray(e.objects) ? (e.objects as number[]) : [] }));
    on('chat.toolDone', (e) => {
      const c = this.chips.get(String(e.id ?? ''));
      if (!c) return;
      this.status(c, String(e.status ?? 'done'));
      if (Array.isArray(e.images) && e.images.length) this.images(c, e.images as string[]);
    });
    on('chat.done', (e) => { this.finish(e); if (e.usage) this.showUsage(e.usage as Usage, -1); });
    on('chat.reset', () => {
      this.log.replaceChildren(this.empty); setHidden(this.empty, false); this.reply = null; this.md = null; this.chips.clear(); this.setBusy(false);
      this.showUsage({ input: 0, output: 0, costUsd: 0, context: 0, window: 0, model: '' }, -1);
    });
  }

  release(): void {
    for (const off of this.off) off();
    this.off.length = 0;
  }

  async load(): Promise<void> {
    if (this.loaded) return;
    this.loaded = true;
    try {
      const s = await this.link.cmd<State>('chat.state');
      for (const m of s.messages) {
        if (m.role === 'user') { this.user(m.text ?? '', m.note ?? ''); continue; }
        this.begin();
        for (const p of m.parts ?? []) {
          if (p.kind === 'text') this.delta(p.text ?? '');
          else this.tool(p);
        }
        this.close(m.error ?? '', m.meta ?? {});
      }
      if (s.usage) this.showUsage(s.usage, -1);
      if (s.busy) { this.begin(); this.setBusy(true); }
    } catch (err) { this.onFail(err); }
  }

  async send(text: string, context = '', note = ''): Promise<void> {
    try { await this.link.cmd('chat.send', { text, context, note }); } catch (err) { this.onFail(err); }
  }

  isBusy(): boolean { return this.busy; }

  private setBusy(on: boolean): void {
    this.busy = on;
    setHidden(q(this.root, '.ff-chat-stop'), !on);
    setText(q(this.root, '.ff-chat-status'), on ? this.deps.t('Claude is working...') : '');
    this.onBusy(on);
  }

  private scroll(): void {
    const l = this.log;
    if (l.scrollHeight - l.scrollTop - l.clientHeight < 80) l.scrollTop = l.scrollHeight;
  }

  private user(text: string, note = ''): void {
    setHidden(this.empty, true);
    const n = el('<div class="ff-chat-msg is-user"><span class="ff-chat-said"></span></div>');
    setText(q(n, '.ff-chat-said'), text);
    if (note) { const m = el('<span class="ff-chat-note"></span>'); setText(m, note); n.appendChild(m); }
    this.log.appendChild(n);
    this.scroll();
  }

  private begin(): void {
    setHidden(this.empty, true);
    this.endText();
    this.reply = el('<div class="ff-chat-msg is-claude"></div>');
    this.log.appendChild(this.reply);
    this.setBusy(true);
    this.scroll();
  }

  private delta(text: string): void {
    if (!this.reply) this.begin();
    if (!this.md) { this.md = new MarkdownStream('ff-chat-text ff-md'); this.reply?.appendChild(this.md.root); }
    this.md.push(text);
    this.scroll();
  }

  private endText(): void {
    this.md?.end();
    this.md = null;
  }

  private images(after: HTMLElement, names: string[]): void {
    const row = el('<div class="ff-chat-images"></div>');
    for (const name of names.slice(0, 6)) {
      const img = el<HTMLImageElement>(`<img alt="${this.deps.t('What Claude looked at')}" decoding="async" draggable="false" width="240" height="180">`);
      img.src = this.link.url(`/api/chat/image/${encodeURIComponent(name)}`);
      row.appendChild(img);
    }
    after.after(row);
    this.scroll();
  }

  private showUsage(u: Usage, reply: number): void {
    const t = this.deps.t;
    this.usage = u;
    if (!u.context && !u.input && reply < 0) { setText(this.meter, ''); return; }
    const parts: string[] = [];
    if (u.context) parts.push(`${t('Memory')} ${tokens(u.context)}${u.window ? ` ${t('of')} ${tokens(u.window)}` : ''}`);
    if (reply >= 0) parts.push(`${t('Reply')} ${tokens(reply)} ${t('tokens')}`);
    if (u.input || u.output) parts.push(`${t('Chat')} ${tokens(u.input + u.output)} ${t('tokens')}${u.costUsd ? ` · $${u.costUsd.toFixed(u.costUsd < 1 ? 3 : 2)}` : ''}`);
    setText(this.meter, parts.join(' · '));
    setClass(this.meter, 'is-high', !!u.window && u.context / u.window > 0.8);
  }

  private async handoff(): Promise<void> {
    const t = this.deps.t;
    try {
      const s = await this.link.cmd<State>('chat.state');
      if (!s.session) { this.onFail(new Error(t('Start a conversation here first; then it can continue in Claude Code'))); return; }
      if (s.busy) { this.onFail(new Error(t('Wait for Claude to finish, or press Stop, before moving the conversation'))); return; }
      this.onHandoff(s.session);
    } catch (err) { this.onFail(err); }
  }

  private async adopt(): Promise<void> {
    const t = this.deps.t;
    try {
      const r = await this.link.cmd<{ sessions: Session[] }>('chat.sessions');
      const first = r.sessions.find((s) => !s.current);
      if (!first) { this.onFail(new Error(t('No other Claude Code conversation in formfabric yet: open one with the Claude Code button first'))); return; }
      const lang = this.deps.language?.() ?? undefined;
      const when = (s: number): string => new Date(s * 1000).toLocaleString(lang, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
      const options = r.sessions.map((s) => {
        const o = el<HTMLOptionElement>('<option></option>');
        o.value = s.id;
        o.textContent = `${when(s.modified)} · ${s.title || t('Untitled')}${s.current ? ` (${t('this chat')})` : ''}`;
        return o;
      });
      this.pickSelect.replaceChildren(...options);
      this.pickSelect.value = first.id;
      setHidden(this.pick, false);
      this.pickSelect.focus();
    } catch (err) { this.onFail(err); }
  }

  private async adoptChosen(): Promise<void> {
    const id = this.pickSelect.value;
    setHidden(this.pick, true);
    if (!id) return;
    try { await this.link.cmd('chat.adopt', { session: id }); } catch (err) { this.onFail(err); }
  }

  private tool(p: Part): void {
    if (!this.reply) this.begin();
    this.endText();
    const chip = el(`<div class="ff-chat-tool" title=""><span class="ff-chat-tool-mark"><span class="icon">${this.deps.icons.icon('terminal', 11)}</span></span><span class="ff-chat-tool-name"></span><span class="ff-chat-tool-input"></span></div>`);
    setText(q(chip, '.ff-chat-tool-name'), p.name ?? '');
    setText(q(chip, '.ff-chat-tool-input'), p.input ?? '');
    chip.title = p.input ?? '';
    if (p.objects?.length) {
      chip.classList.add('has-objects');
      chip.addEventListener('click', () => this.onObjects(p.objects ?? []));
    }
    this.status(chip, p.status ?? 'running');
    if (p.id) this.chips.set(p.id, chip);
    this.reply?.appendChild(chip);
    if (p.images?.length) this.images(chip, p.images);
    this.scroll();
  }

  private status(chip: HTMLElement, s: string): void {
    for (const k of ['running', 'done', 'error']) setClass(chip, `is-${k}`, s === k);
  }

  private finish(e: EngineEvent): void {
    this.close(String(e.error ?? ''), (e.meta ?? {}) as Meta);
  }

  private close(error: string, meta: Meta): void {
    if (!this.reply) this.begin();
    this.endText();
    for (const c of this.reply?.querySelectorAll<HTMLElement>('.ff-chat-tool.is-running') ?? []) this.status(c, 'done');
    if (error) {
      const n = el('<div class="ff-chat-error"></div>');
      setText(n, error);
      this.reply?.appendChild(n);
    }
    if (meta.seconds !== undefined) {
      const n = el('<div class="ff-chat-meta"></div>');
      const used = meta.inputTokens || meta.outputTokens ? ` · ${tokens((meta.inputTokens ?? 0) + (meta.outputTokens ?? 0))} ${this.deps.t('tokens')}` : '';
      setText(n, `${meta.seconds} s${meta.turns ? ` · ${meta.turns} ${this.deps.t('steps')}` : ''}${used}${meta.costUsd ? ` · $${meta.costUsd.toFixed(3)}` : ''}`);
      this.reply?.appendChild(n);
    }
    this.reply = null;
    this.setBusy(false);
    this.scroll();
  }
}
