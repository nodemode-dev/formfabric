import type { DockAdapter, DockAgentStatus, DockChannel, DockErrorKind, DockTerminal } from './dock/adapter';
import type { Dock } from './dock/dock';
import { AgentLink } from './agent/agent-link';
import { ChatPane } from './agent/chat-pane';
import { Link, ServiceError, type DockDeps } from './agent/link';

export type FabricKind = 'antigravity';
type Terminal = DockTerminal<FabricKind>;

const EXIT = '{"type": "exit"}';
const LEAD = "Ask Claude about your garment in the line below: draw panels, shape curves, add inner lines or check measurements. It works through formfabric's tools, every change shows here and in the views, and each one can be undone.";

function errorKind(err: unknown): DockErrorKind | null {
  if (err instanceof TypeError) return 'unreachable';
  if (!(err instanceof ServiceError)) return null;
  if (err.status === 403) return 'refused';
  if (err.status === 404 && /^Unknown command/.test(err.message)) return 'oldEngine';
  if (/not installed|was not found\. Looked in|but does not start/i.test(err.message)) return 'notInstalled';
  if (err.status === 409) return 'busy';
  return err.status === 502 || err.status === 503 || err.status === 504 ? 'unreachable' : null;
}

function channel(link: Link, id: string): DockChannel {
  const socket = new WebSocket(link.socketUrl(`/api/terminal/${encodeURIComponent(id)}`));
  let data = (_chunk: string | Uint8Array): void => undefined;
  let exit = (): void => undefined;
  let closed = (): void => undefined;
  const waiting: string[] = [];
  socket.addEventListener('open', () => {
    link.handshake(socket);
    for (const text of waiting.splice(0)) socket.send(text);
  });
  socket.addEventListener('message', (msg) => {
    const text = String(msg.data);
    if (text === EXIT) exit();
    else data(text);
  });
  socket.addEventListener('close', () => closed());
  const send = (text: string): void => {
    if (socket.readyState === WebSocket.OPEN) socket.send(text);
    else if (socket.readyState === WebSocket.CONNECTING) waiting.push(text);
  };
  return {
    send: (chunk) => send(JSON.stringify({ type: 'input', data: chunk })),
    resize: (cols, rows) => send(JSON.stringify({ type: 'resize', cols, rows })),
    onData: (handler) => { data = handler; },
    onExit: (handler) => { exit = handler; },
    onClose: (handler) => { closed = handler; },
    close: () => socket.close(),
  };
}

export interface FabricDock { adapter: DockAdapter<FabricKind>; attach(dock: Dock<FabricKind>): void }

export function formfabricDock(deps: DockDeps, openSettings: () => void): FabricDock {
  const link = new Link(deps.serviceUrl);
  const outside = new AgentLink(document.body, link, deps);
  let dock: Dock<FabricKind> | null = null;
  link.start();
  outside.start();
  const adapter: DockAdapter<FabricKind> = {
    words: { app: 'formfabric', noun: 'garment', lead: LEAD, example: 'draw a front bodice panel 40 cm wide', start: './formfabric' },
    placement: {
      mode: 'own',
      storage: { key: 'formfabric.dock' },
      sizes: { minHeight: 140, maxHeightShare: 0.7, defaultHeight: 280, minWidth: 320, maxWidthShare: 0.6, defaultWidth: 440 },
    },
    tabs: () => ({ builtin: ['chat', 'shell', 'claude', 'codex', 'cursor', 'antigravity'] }),
    classes: { iconButton: 'ff-tt-btn', button: 'ff-btn', primaryButton: 'ff-btn is-primary', input: 'ff-input', select: 'ff-input', note: 'ff-note' },
    t: (_key, english) => deps.t(english),
    onTheme: (handler) => { document.addEventListener('nm:ui-settings-changed', handler); },
    errors: { message: (err) => (err instanceof ServiceError ? err.message : null), kind: errorKind },
    terminal: {
      list: async () => (await link.cmd<{ terminals: Terminal[] }>('terminal.list')).terminals,
      open: async (kind, o) => {
        if (kind === 'shell') return link.cmd<Terminal>('terminal.open', { cols: o.cols, rows: o.rows });
        if (kind === 'claude') return link.cmd<Terminal>('claudecode.open', { cols: o.cols, rows: o.rows, ...(o.prompt ? { prompt: o.prompt } : {}), ...(o.resume ? { resume: o.resume } : {}) });
        return link.cmd<Terminal>('terminal.agent', { kind, cols: o.cols, rows: o.rows });
      },
      restart: async (id) => { await link.cmd('terminal.restart', { id }); },
      close: async (id) => { await link.cmd('terminal.close', { id }); },
      channel: (id) => channel(link, id),
    },
    agentStatus: async () => {
      try { return await link.cmd<DockAgentStatus<FabricKind>>('agents.status'); } catch (err) {
        if (errorKind(err) !== 'oldEngine') return null;
        const old = await link.cmd<{ installed: boolean; version: string | null }>('claudecode.status').catch(() => null);
        return old ? { claude: { installed: old.installed, version: old.version, install: '', signedIn: null } } : null;
      }
    },
    chat: {
      ask: 'line',
      pane: (host) => {
        const pane = new ChatPane(link, deps);
        pane.onFail = (err) => deps.onFail?.(err);
        pane.onHandoff = (session) => { void dock?.claude(undefined, session); };
        host.appendChild(pane.root);
        pane.root.hidden = false;
        return {
          send: (text) => {
            if (pane.isBusy()) return false;
            void pane.send(text, deps.context?.() ?? '', deps.note?.() ?? '');
            return true;
          },
          shown: () => { void pane.load(); },
          busy: () => pane.isBusy(),
        };
      },
    },
    openSettings,
  };
  return {
    adapter,
    attach: (d) => { dock = d; },
  };
}
