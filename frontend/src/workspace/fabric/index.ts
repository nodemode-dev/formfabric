import './fabric.css';
import type { ShellDeps } from '../../shell/deps';
import type { DocStore } from '../doc';
import type { Selection } from '../state';
import { FabricPanel } from './panel';

export function mountFabric(root: HTMLElement, deps: ShellDeps, doc: DocStore, sel: Selection): void {
  new FabricPanel(root, deps, doc, sel);
}
