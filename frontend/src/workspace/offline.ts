import { onEngineLink } from '../engine/service';
import { setText } from '../ui/dom';
import { onLanguage, t } from '../ui/i18n';

export const OFFLINE = 'The engine is not running. Start it with ./formfabric';

let down = false;
const subs = new Set<() => void>();

onEngineLink((state) => {
  const next = state === 'down';
  if (next === down) return;
  down = next;
  for (const fn of subs) fn();
});

export function engineDown(): boolean {
  return down;
}

export function onEngineDown(fn: () => void): void {
  subs.add(fn);
}

export function emptyLabel(node: HTMLElement, key: string): void {
  const paint = (): void => setText(node, t(down ? OFFLINE : key));
  paint();
  onLanguage(paint);
  subs.add(paint);
}
