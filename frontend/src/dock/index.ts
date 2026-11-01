import { AgentLink } from './agent-link';
import { Link, type DockDeps } from './link';
import { TerminalDock, type DockKind } from './terminal-dock';

export type { DockDeps, DockKind };

export interface DockHandle {
  isOpen(): boolean;
  toggle(on?: boolean): void;
  show(kind: DockKind): Promise<void>;
  toggleKind(kind: DockKind): Promise<void>;
  newTerminal(): Promise<void>;
  claude(prompt?: string): Promise<void>;
  focus(): void;
  onChange(cb: () => void): void;
  onAgentChange(cb: () => void): void;
  destroy(): void;
}

function seed(): void {
  const html = document.documentElement;
  let open = false;
  let h = 280;
  try {
    const v = JSON.parse(localStorage.getItem('formfabric.dock') ?? '{}') as { open?: boolean; h?: number };
    open = v.open === true;
    if (Number.isFinite(v.h) && (v.h as number) >= 140) h = v.h as number;
  } catch { open = false; }
  if (!html.hasAttribute('data-ft-dock')) html.setAttribute('data-ft-dock', open ? 'open' : 'closed');
  if (!html.hasAttribute('data-ft-dock-h')) html.setAttribute('data-ft-dock-h', String(h));
  if (!html.style.getPropertyValue('--ft-dock-size')) html.style.setProperty('--ft-dock-size', `${h}px`);
}

export function mountDock(host: HTMLElement, deps: DockDeps): DockHandle {
  seed();
  const link = new Link(deps.serviceUrl);
  const dock = new TerminalDock(host, link, deps);
  dock.onFail = (err) => deps.onFail?.(err);
  dock.context = () => deps.context?.() ?? '';
  dock.note = () => deps.note?.() ?? '';
  const agent = new AgentLink(document.body, link, deps);
  let changed: () => void = () => undefined;
  agent.onChange = () => changed();
  link.start();
  agent.start();
  return {
    isOpen: () => dock.isOpen(),
    toggle: (on) => dock.toggle(on),
    show: (kind) => dock.show(kind),
    toggleKind: (kind) => dock.toggleKind(kind),
    newTerminal: () => dock.newTerminal(),
    claude: async (prompt) => {
      if (!dock.isOpen()) dock.toggle(true, 'none');
      await dock.claude(prompt);
    },
    focus: () => dock.focus(),
    onChange: (cb) => { dock.onChange = cb; },
    onAgentChange: (cb) => { changed = cb; },
    destroy: () => {
      agent.stop();
      link.stop();
      dock.destroy();
    },
  };
}
