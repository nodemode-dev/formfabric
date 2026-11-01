import './body.css';
import type { ShellDeps } from '../../shell/deps';
import type { DocStore } from '../doc';
import type { Selection } from '../state';
import { BodyPanel } from './panel';

export function mountBody(root: HTMLElement, deps: ShellDeps, doc: DocStore, sel: Selection): void {
  new BodyPanel(root, deps, doc, sel);
}
