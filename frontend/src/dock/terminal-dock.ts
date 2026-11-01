import { FitAddon } from '@xterm/addon-fit';
import { Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import '../styles/dock.css';
import { ChatPane } from './chat-pane';
import { escapeHtml, fromHtml as el, isMac, q, setDisabled, setHidden, setText } from './dom';
import type { DockDeps, Link } from './link';

export type DockKind = 'shell' | 'claude' | 'chat';

export interface TermInfo { id: string; title: string; kind?: string; alive?: boolean }

interface Tab {
  id: string;
  term: Terminal;
  fit: FitAddon;
  host: HTMLElement;
  tab: HTMLElement;
  socket: WebSocket | null;
  kind: string;
}

const KEY = 'formfabric.dock';
const MIN_H = 140;

function saved(): { open: boolean; h: number } {
  try {
    const v = JSON.parse(localStorage.getItem(KEY) ?? '{}') as { open?: boolean; h?: number };
    return { open: v.open === true, h: Number.isFinite(v.h) && (v.h as number) >= MIN_H ? (v.h as number) : 280 };
  } catch { return { open: false, h: 280 }; }
}

function keep(open: boolean, h: number): void {
  try { localStorage.setItem(KEY, JSON.stringify({ open, h })); } catch { return; }
}

function token(name: string, fallback: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
}

function isToggle(e: KeyboardEvent): boolean {
  return (isMac ? e.metaKey : e.ctrlKey) && !e.altKey && !e.shiftKey && e.code === 'KeyJ';
}

export class TerminalDock {
  readonly root: HTMLElement;
  onFail: (err: unknown) => void = () => undefined;
  context: () => string = () => '';
  note: () => string = () => '';
  onObjects: (ids: number[]) => void = () => undefined;
  onChange: () => void = () => undefined;
  private readonly tabsEl: HTMLElement;
  private readonly stage: HTMLElement;
  private readonly empty: HTMLElement;
  private readonly ask: HTMLInputElement;
  private readonly restartBtn: HTMLButtonElement;
  private readonly tabs = new Map<string, Tab>();
  private readonly watchers: Array<() => void> = [];
  private active: string | null = null;
  private chat: ChatPane | null = null;
  private chatTab: HTMLElement | null = null;
  private started = false;
  private pending: Promise<void> | null = null;

  constructor(host: HTMLElement, private readonly link: Link, private readonly deps: DockDeps) {
    const t = deps.t;
    const icon = (name: string, size: number): string => `<span class="icon">${deps.icons.icon(name, size)}</span>`;
    const mod = isMac ? 'Cmd' : 'Ctrl';
    this.root = host;
    host.classList.add('ff-dock');
    host.setAttribute('aria-label', t('Terminal'));
    host.append(
      el(`<div class="ff-dock-grip" role="separator" aria-orientation="horizontal" aria-label="${t('Resize the terminal')}"></div>`),
      el(`<div class="ff-dock-head">
<div class="ff-dock-tabs" role="tablist" aria-label="${t('Terminals')}"></div>
<span class="ff-dock-spacer"></span>
<button type="button" class="ff-tt-btn ff-dock-new" title="${t('New Terminal')}" aria-label="${t('New Terminal')}">${icon('plus', 15)}</button>
<button type="button" class="ff-tt-btn ff-dock-chat" title="${t('Chat with Claude about your garment')}" aria-label="${t('Chat with Claude')}">${icon('agentChat', 15)}</button>
<button type="button" class="ff-tt-btn ff-dock-claude" title="${t('Claude Code')}" aria-label="${t('Claude Code')}">${icon('agent', 15)}</button>
<button type="button" class="ff-tt-btn ff-dock-restart" title="${t('Restart this tab')}" aria-label="${t('Restart this tab')}">${icon('refresh', 15)}</button>
<button type="button" class="ff-tt-btn ff-dock-hide" title="${t('Hide Terminal')} (${mod}+J); ${t('programs keep running')}" aria-label="${t('Hide Terminal')}">${icon('close', 15)}</button>
</div>`),
      el(`<div class="ff-dock-stage"><p class="ff-dock-empty ff-note" hidden>${t('No terminal is open. Start a shell with the plus button, Claude Code with the agent button, or Chat with the speech bubble.')}</p></div>`),
      el(`<form class="ff-dock-ask" autocomplete="off">
<span class="ff-dock-ask-mark">${icon('agent', 15)}</span>
<input class="ff-input ff-dock-ask-input" type="text" spellcheck="false" placeholder="${t('Ask Claude Code to draft your pattern')}" aria-label="${t('Ask Claude Code')}">
<button type="submit" class="ff-tt-btn" title="${t('Send to Claude Code')}" aria-label="${t('Send to Claude Code')}">${icon('send', 14)}</button>
</form>`),
    );
    this.tabsEl = q(host, '.ff-dock-tabs');
    this.stage = q(host, '.ff-dock-stage');
    this.empty = q(host, '.ff-dock-empty');
    this.ask = q<HTMLInputElement>(host, '.ff-dock-ask-input');
    this.restartBtn = q<HTMLButtonElement>(host, '.ff-dock-restart');
    q(host, '.ff-dock-new').addEventListener('click', () => void this.open());
    q(host, '.ff-dock-claude').addEventListener('click', () => void this.claude());
    q(host, '.ff-dock-chat').addEventListener('click', () => void this.showChat());
    q(host, '.ff-dock-hide').addEventListener('click', () => this.toggle(false));
    this.restartBtn.addEventListener('click', () => void this.restart());
    q(host, '.ff-dock-ask').addEventListener('submit', (e) => { e.preventDefault(); void this.send(); });
    host.addEventListener('keydown', (e) => { if (!isToggle(e)) e.stopPropagation(); });
    const ro = new ResizeObserver(() => this.fitActive());
    ro.observe(this.stage);
    this.watchers.push(() => ro.disconnect());
    const onKey = (e: KeyboardEvent): void => {
      if (e.repeat || !(isMac ? e.metaKey : e.ctrlKey) || e.code !== 'KeyJ') return;
      if (e.altKey && !e.shiftKey) { e.preventDefault(); e.stopPropagation(); void this.toggleKind('chat'); }
      else if (e.shiftKey && !e.altKey) { e.preventDefault(); e.stopPropagation(); void this.toggleKind('claude'); }
      else if (!e.shiftKey && !e.altKey) { e.preventDefault(); e.stopPropagation(); this.toggle(); }
    };
    document.addEventListener('keydown', onKey, true);
    this.watchers.push(() => document.removeEventListener('keydown', onKey, true));
    const onSettings = (): void => {
      for (const x of this.tabs.values()) x.term.options.theme = this.theme();
      this.fitActive();
    };
    document.addEventListener('nm:ui-settings-changed', onSettings);
    this.watchers.push(() => document.removeEventListener('nm:ui-settings-changed', onSettings));
    this.grip(q(host, '.ff-dock-grip'));
    setDisabled(this.restartBtn, true);
    if (this.isOpen()) void this.ensure();
  }

  isOpen(): boolean { return document.documentElement.getAttribute('data-ft-dock') === 'open'; }

  toggle(on = !this.isOpen(), seed: DockKind | 'none' = 'shell'): void {
    const html = document.documentElement;
    html.setAttribute('data-ft-dock', on ? 'open' : 'closed');
    keep(on, this.height());
    this.onChange();
    window.dispatchEvent(new Event('resize'));
    if (on) void this.ensure(seed);
  }

  activeKind(): DockKind | null {
    if (this.active === 'chat') return 'chat';
    const kind = this.active ? this.tabs.get(this.active)?.kind : undefined;
    return kind === 'shell' || kind === 'claude' ? kind : null;
  }

  async show(kind: DockKind): Promise<void> {
    if (!this.isOpen()) this.toggle(true, 'none');
    await this.ensure('none');
    if (kind === 'chat') { await this.showChat(); return; }
    const own = [...this.tabs.values()].find((x) => x.kind === kind);
    if (own) this.select(own.id);
    else if (kind === 'claude') await this.claude();
    else await this.open();
  }

  async toggleKind(kind: DockKind): Promise<void> {
    if (this.isOpen() && this.activeKind() === kind) this.toggle(false);
    else await this.show(kind);
  }

  async newTerminal(): Promise<void> {
    if (!this.isOpen()) this.toggle(true);
    else await this.open();
  }

  destroy(): void {
    for (const stop of this.watchers) stop();
    this.watchers.length = 0;
    for (const entry of this.tabs.values()) { entry.socket?.close(); entry.term.dispose(); }
    this.tabs.clear();
    this.chat?.release();
  }

  private height(): number {
    const h = Number(document.documentElement.getAttribute('data-ft-dock-h'));
    return Number.isFinite(h) && h >= MIN_H ? h : saved().h;
  }

  private grip(grip: HTMLElement): void {
    const html = document.documentElement;
    grip.addEventListener('pointerdown', (e) => {
      e.preventDefault();
      grip.setPointerCapture(e.pointerId);
      const bottom = this.root.getBoundingClientRect().bottom;
      const move = (ev: PointerEvent): void => {
        const h = Math.round(Math.min(window.innerHeight * 0.7, Math.max(MIN_H, bottom - ev.clientY)));
        html.setAttribute('data-ft-dock-h', String(h));
        html.style.setProperty('--ft-dock-size', `${h}px`);
      };
      const up = (): void => {
        grip.removeEventListener('pointermove', move);
        grip.removeEventListener('pointerup', up);
        grip.removeEventListener('pointercancel', up);
        keep(this.isOpen(), this.height());
        window.dispatchEvent(new Event('resize'));
      };
      grip.addEventListener('pointermove', move);
      grip.addEventListener('pointerup', up);
      grip.addEventListener('pointercancel', up);
    });
  }

  private ensure(seed: DockKind | 'none' = 'shell'): Promise<void> {
    this.pending ??= this.boot(seed).finally(() => { this.pending = null; });
    return this.pending;
  }

  private async boot(seed: DockKind | 'none'): Promise<void> {
    if (!this.started) {
      this.started = true;
      const listed = await this.link.cmd<{ terminals: TermInfo[] }>('terminal.list').catch(() => ({ terminals: [] as TermInfo[] }));
      for (const info of listed.terminals) if (!this.tabs.has(info.id)) this.mount(info, false);
    }
    if (!this.tabs.size && !this.chatTab && seed === 'shell') await this.open();
    else if (!this.active && this.tabs.size) this.select([...this.tabs.keys()][0]);
    setHidden(this.empty, this.tabs.size > 0 || !!this.chatTab);
    this.fitActive();
    this.focus();
  }

  focus(): void {
    if (this.active === 'chat') this.ask.focus({ preventScroll: true });
    else if (this.active) this.tabs.get(this.active)?.term.focus();
  }

  async showChat(): Promise<void> {
    const icon = (name: string, size: number): string => `<span class="icon">${this.deps.icons.icon(name, size)}</span>`;
    if (!this.chat) {
      const chat = new ChatPane(this.link, this.deps);
      chat.onFail = (err) => this.onFail(err);
      chat.onHandoff = (session) => void this.claude(undefined, session);
      chat.onObjects = (ids) => this.onObjects(ids);
      this.chat = chat;
      this.stage.appendChild(chat.root);
    }
    if (!this.chatTab) {
      const tab = el(`<button type="button" class="ff-dock-tab is-chat" role="tab" aria-selected="false">${icon('agentChat', 13)}<span class="ff-dock-tab-title">${this.deps.t('Chat')}</span><span class="ff-dock-tab-x" role="button" aria-label="${this.deps.t('Close the Chat tab')}">${icon('close', 11)}</span></button>`);
      tab.addEventListener('click', (e) => {
        if ((e.target as HTMLElement).closest('.ff-dock-tab-x')) this.closeChat();
        else this.select('chat');
      });
      this.tabsEl.prepend(tab);
      this.chatTab = tab;
    }
    setHidden(this.empty, true);
    this.select('chat');
    await this.chat.load();
  }

  private closeChat(): void {
    this.chatTab?.remove();
    this.chatTab = null;
    if (this.chat) setHidden(this.chat.root, true);
    if (this.active === 'chat') {
      this.active = null;
      const next = [...this.tabs.keys()].pop();
      if (next) this.select(next);
      else { setHidden(this.empty, false); this.syncAsk(); setDisabled(this.restartBtn, true); }
    }
  }

  private syncAsk(): void {
    const want = this.deps.t(this.active === 'chat' ? 'Ask Claude about your garment (Enter to send)' : 'Ask Claude Code to draft your pattern');
    if (this.ask.placeholder !== want) this.ask.placeholder = want;
  }

  async open(): Promise<void> {
    try {
      const info = await this.link.cmd<TermInfo>('terminal.open', { cols: 100, rows: 20 });
      this.mount(info, true);
    } catch (err) { this.onFail(err); }
  }

  async claude(prompt?: string, resume?: string): Promise<void> {
    const size = this.size();
    try {
      const info = await this.link.cmd<TermInfo>('claudecode.open', { prompt: prompt || undefined, cols: size.cols, rows: size.rows, resume: resume || undefined });
      if (this.tabs.has(info.id)) this.select(info.id);
      else this.mount(info, true);
    } catch (err) { this.onFail(err); }
  }

  private async send(): Promise<void> {
    const text = this.ask.value.trim();
    if (!text) return;
    const extra = this.context();
    if (this.active === 'chat' && this.chat?.isBusy()) return;
    this.ask.value = '';
    if (this.active === 'chat' && this.chat) await this.chat.send(text, extra, this.note());
    else await this.claude(extra ? `${text} (${extra})` : text);
  }

  private size(): { cols: number; rows: number } {
    const tab = this.active ? this.tabs.get(this.active) : null;
    return tab ? { cols: tab.term.cols, rows: tab.term.rows } : { cols: 100, rows: 20 };
  }

  private theme() {
    return {
      background: token('--bg-primary', '#111111'),
      foreground: token('--text-primary', '#f7f7f7'),
      cursor: token('--accent-color', '#f97316'),
      cursorAccent: token('--bg-primary', '#111111'),
      selectionBackground: token('--accent-color-dim', '#f9731633'),
    };
  }

  private mount(info: TermInfo, show: boolean): void {
    const kind = info.kind ?? 'shell';
    const icon = (name: string, size: number): string => `<span class="icon">${this.deps.icons.icon(name, size)}</span>`;
    const term = new Terminal({
      fontFamily: token('--font-mono', 'monospace'), fontSize: 12, lineHeight: 1.2, cursorBlink: true,
      scrollback: 5000, allowProposedApi: false, theme: this.theme(), overviewRuler: { width: 0 },
    });
    term.attachCustomKeyEventHandler((e) => !((isMac ? e.metaKey : e.ctrlKey) && !e.altKey && e.code === 'KeyJ'));
    const fit = new FitAddon();
    term.loadAddon(fit);
    const host = el(`<div class="ff-term" data-id="${escapeHtml(info.id)}" hidden></div>`);
    const tab = el(`<button type="button" class="ff-dock-tab${kind === 'claude' ? ' is-claude' : ''}" role="tab" aria-selected="false">${icon(kind === 'claude' ? 'agent' : 'terminal', 13)}<span class="ff-dock-tab-title" data-no-i18n></span><span class="ff-dock-tab-x" role="button" aria-label="${this.deps.t('Close this tab')}">${icon('close', 11)}</span></button>`);
    setText(q(tab, '.ff-dock-tab-title'), info.title);
    tab.addEventListener('click', (e) => {
      if ((e.target as HTMLElement).closest('.ff-dock-tab-x')) void this.close(info.id);
      else this.select(info.id);
    });
    this.stage.appendChild(host);
    this.tabsEl.appendChild(tab);
    const entry: Tab = { id: info.id, term, fit, host, tab, socket: null, kind };
    this.tabs.set(info.id, entry);
    setHidden(this.empty, true);
    term.open(host);
    if (show || !this.active) this.select(info.id);
    this.connect(entry);
  }

  private connect(entry: Tab): void {
    const socket = new WebSocket(this.link.socketUrl(`/api/terminal/${encodeURIComponent(entry.id)}`));
    entry.socket = socket;
    socket.addEventListener('open', () => this.sendSize(entry));
    socket.addEventListener('message', (msg) => {
      const data = String(msg.data);
      if (data === '{"type": "exit"}') {
        entry.term.write(`\r\n\x1b[2m[${entry.kind === 'claude' ? 'Claude Code' : 'shell'} ${this.deps.t('exited')} - ${this.deps.t('Restart runs it again')}]\x1b[0m\r\n`);
        return;
      }
      entry.term.write(data);
    });
    entry.term.onData((data) => {
      if (socket.readyState === WebSocket.OPEN) socket.send(JSON.stringify({ type: 'input', data }));
    });
  }

  private sendSize(entry: Tab): void {
    if (entry.socket?.readyState === WebSocket.OPEN) entry.socket.send(JSON.stringify({ type: 'resize', cols: entry.term.cols, rows: entry.term.rows }));
  }

  fitActive(): void {
    const entry = this.active ? this.tabs.get(this.active) : null;
    if (!entry || entry.host.hidden || !this.isOpen() || this.stage.clientHeight < 20) return;
    const before = `${entry.term.cols}x${entry.term.rows}`;
    entry.fit.fit();
    if (`${entry.term.cols}x${entry.term.rows}` !== before) this.sendSize(entry);
  }

  private select(id: string): void {
    this.active = id;
    for (const [tid, entry] of this.tabs) {
      const on = tid === id;
      setHidden(entry.host, !on);
      entry.tab.classList.toggle('is-on', on);
      entry.tab.setAttribute('aria-selected', String(on));
    }
    if (this.chat) setHidden(this.chat.root, id !== 'chat');
    if (this.chatTab) { this.chatTab.classList.toggle('is-on', id === 'chat'); this.chatTab.setAttribute('aria-selected', String(id === 'chat')); }
    this.syncAsk();
    setDisabled(this.restartBtn, id === 'chat');
    this.fitActive();
    this.focus();
  }

  private async restart(): Promise<void> {
    const entry = this.active ? this.tabs.get(this.active) : null;
    if (!entry) return;
    try {
      await this.link.cmd('terminal.restart', { id: entry.id });
      this.sendSize(entry);
      entry.term.focus();
    } catch (err) { this.onFail(err); }
  }

  private async close(id: string): Promise<void> {
    const entry = this.tabs.get(id);
    if (!entry) return;
    entry.socket?.close();
    entry.term.dispose();
    entry.host.remove();
    entry.tab.remove();
    this.tabs.delete(id);
    await this.link.cmd('terminal.close', { id }).catch(() => undefined);
    if (this.active === id) {
      this.active = null;
      const next = [...this.tabs.keys()].pop() ?? (this.chatTab ? 'chat' : undefined);
      if (next) this.select(next);
      else setDisabled(this.restartBtn, true);
    }
    setHidden(this.empty, this.tabs.size > 0 || !!this.chatTab);
  }
}
