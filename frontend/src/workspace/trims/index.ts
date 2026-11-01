import type { ShellDeps } from '../../shell/deps';
import type { DocStore } from '../doc';
import type { Selection } from '../state';
import { TrimsPanel } from './panel';

export function mountTrims(root: HTMLElement, deps: ShellDeps, doc: DocStore, sel: Selection): void {
  new TrimsPanel(root, deps, doc, sel);
}
