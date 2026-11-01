import { openStream, type Stream } from '../dock/sse';

export interface EngineChanged { type: 'engine.changed'; epoch: number; name: string }
export type Link = 'connecting' | 'up' | 'down';

type Listener = (e: EngineChanged) => void;
type LinkListener = (state: Link) => void;

const listeners = new Set<Listener>();
const linkListeners = new Set<LinkListener>();
let source: Stream | null = null;
let link: Link = 'connecting';

function eventsUrl(): string {
  const injected = (window as unknown as { __FFB_ENGINE__?: string }).__FFB_ENGINE__;
  return `${injected ? injected.replace(/\/$/, '') : ''}/api/events`;
}

function setLink(next: Link): void {
  if (next === link) return;
  link = next;
  for (const fn of linkListeners) fn(next);
}

function open(): void {
  if (source || !(listeners.size || linkListeners.size)) return;
  source = openStream(eventsUrl(), {
    open: () => setLink('up'),
    error: (refused) => setLink(refused ? 'down' : 'connecting'),
    message: (text) => {
      setLink('up');
      try {
        const data = JSON.parse(text) as Partial<EngineChanged>;
        if (data.type !== 'engine.changed') return;
        const e: EngineChanged = { type: 'engine.changed', epoch: Number(data.epoch ?? 0), name: String(data.name ?? '') };
        for (const fn of listeners) fn(e);
      } catch { return; }
    },
  });
}

export function onEngineChanged(cb: Listener): () => void {
  listeners.add(cb);
  open();
  return () => { listeners.delete(cb); };
}

export function onEngineLink(cb: LinkListener): () => void {
  linkListeners.add(cb);
  cb(link);
  open();
  return () => { linkListeners.delete(cb); };
}
