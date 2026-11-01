import type { DocStore } from '../doc';
import type { Garment } from '../types';
import { findPrint, num, styleOf, trimDoc, type TrimItem } from './types';

const REMOVE: Record<TrimItem['kind'], [string, string]> = {
  hardware: ['hardware.delete', 'hardware'],
  zipper: ['zipper.delete', 'zipper'],
  stitch: ['stitch.delete', 'stitch'],
  finish: ['finish.delete', 'finish'],
  print: ['print.delete', 'print'],
};

export async function removeTrim(doc: DocStore, item: TrimItem): Promise<void> {
  const [command, key] = REMOVE[item.kind];
  await doc.run(command, { [key]: item.id });
}

export function itemLabel(g: Garment, item: TrimItem): string {
  const d = trimDoc(g);
  if (item.kind === 'hardware') {
    const h = d.hardware.find((x) => x.id === item.id);
    const kind = h ? styleOf(d, h.style)?.kind : undefined;
    return kind === 'buttonhole' ? 'Buttonhole' : kind === 'trim' ? 'Trim' : 'Button';
  }
  if (item.kind === 'zipper') return 'Zipper';
  if (item.kind === 'stitch') {
    const s = d.stitches.find((x) => x.id === item.id);
    const style = s ? styleOf(d, s.style) : undefined;
    return num(style?.params, 'fray', 0) > 0 ? 'Fray' : style?.kind === 'puckering' ? 'Puckering' : 'Topstitch';
  }
  if (item.kind === 'finish') return 'Edge Finish';
  return findPrint(g, item.id) ? 'Print' : 'Print';
}
