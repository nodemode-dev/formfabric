import '../fabric/fabric.css';
import './output.css';
import type { ShellDeps } from '../../shell/deps';
import type { DocStore } from '../doc';
import type { PatternView } from '../pattern-view/view';
import type { Selection } from '../state';
import { OutputPanel } from './panel';

export function mountOutput(root: HTMLElement, deps: ShellDeps, doc: DocStore, sel: Selection, view: PatternView): void {
  new OutputPanel(root, deps, doc, sel, view);
}
