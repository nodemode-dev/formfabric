import type { ShellDeps } from '../shell/deps';
import { assemble } from './layout';

export function mountWorkspace(host: HTMLElement, deps: ShellDeps): void {
  assemble(host, deps);
}
