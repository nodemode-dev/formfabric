import type { SeamEnd } from '../pattern-view/sew-types';
import type { Garment, Panel, Vec } from '../types';

export type StyleKind = 'button' | 'buttonhole' | 'zipper' | 'topstitch' | 'puckering' | 'trim';
export type FinishKind = 'piping' | 'binding' | 'elastic' | 'tape' | 'bond' | 'press';
export type ItemKind = 'hardware' | 'zipper' | 'stitch' | 'finish' | 'print';
export type PrintSide = 'auto' | 'front' | 'back';

export const STYLE_KINDS: StyleKind[] = ['button', 'buttonhole', 'zipper', 'topstitch', 'puckering', 'trim'];
export const FINISH_KINDS: FinishKind[] = ['piping', 'binding', 'elastic', 'tape', 'bond', 'press'];

export interface TStyle { id: number; kind: StyleKind; name: string; params: Record<string, unknown> }
export interface THardware { id: number; style: number; anchor: { panel: number; point: Vec }; rotation: number; fastened_to: number | null }
export interface TZipper { id: number; style: number; a: SeamEnd[]; b: SeamEnd[]; closed: number }
export interface TStitch { id: number; style: number; spans: SeamEnd[]; offset: number }
export interface TFinish { id: number; kind: FinishKind; spans: SeamEnd[]; width: number; ratio: number; strength: number; fabric: number | null; generated: number[] }
export interface TPrint { id: number; name: string; image: string; position: Vec; rotation: number; size: Vec; side: PrintSide; emboss: number }

export interface TrimDoc {
  styles: TStyle[];
  hardware: THardware[];
  zippers: TZipper[];
  stitches: TStitch[];
  finishes: TFinish[];
}

export interface TrimItem { kind: ItemKind; id: number }

type WithTrims = Garment & Partial<TrimDoc>;
type WithPrints = Panel & { prints?: TPrint[] };

const memo = new WeakMap<Garment, TrimDoc>();

export function trimDoc(g: Garment): TrimDoc {
  const hit = memo.get(g);
  if (hit) return hit;
  const w = g as WithTrims;
  const doc: TrimDoc = { styles: w.styles ?? [], hardware: w.hardware ?? [], zippers: w.zippers ?? [], stitches: w.stitches ?? [], finishes: w.finishes ?? [] };
  memo.set(g, doc);
  return doc;
}

export function printsOf(panel: Panel): TPrint[] {
  return (panel as WithPrints).prints ?? [];
}

export function findPrint(g: Garment, id: number): { panel: Panel; print: TPrint } | null {
  for (const panel of g.panels) {
    const print = printsOf(panel).find((p) => p.id === id);
    if (print) return { panel, print };
  }
  return null;
}

export function num(params: Record<string, unknown> | undefined, key: string, fallback: number): number {
  const v = params?.[key];
  return typeof v === 'number' && Number.isFinite(v) ? v : fallback;
}

export function str(params: Record<string, unknown> | undefined, key: string, fallback: string): string {
  const v = params?.[key];
  return typeof v === 'string' ? v : fallback;
}

export function flag(params: Record<string, unknown> | undefined, key: string, fallback: boolean): boolean {
  const v = params?.[key];
  return typeof v === 'boolean' ? v : fallback;
}

export function styleOf(doc: TrimDoc, id: number): TStyle | undefined {
  return doc.styles.find((s) => s.id === id);
}

export function itemExists(g: Garment, item: TrimItem): boolean {
  const d = trimDoc(g);
  if (item.kind === 'hardware') return d.hardware.some((x) => x.id === item.id);
  if (item.kind === 'zipper') return d.zippers.some((x) => x.id === item.id);
  if (item.kind === 'stitch') return d.stitches.some((x) => x.id === item.id);
  if (item.kind === 'finish') return d.finishes.some((x) => x.id === item.id);
  return findPrint(g, item.id) !== null;
}
