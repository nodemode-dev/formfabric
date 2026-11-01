import type { DocStore } from '../doc';
import type { Hit } from '../geom';
import type { Selection } from '../state';
import type { Vec } from '../types';

export interface Cam { x: number; y: number; s: number }

export interface Ptr { sx: number; sy: number; w: Vec; shift: boolean; alt: boolean; mod: boolean; button: number }

export interface Colors { bg: string; grid: string; text: string; muted: string; accent: string; line: string; warn: string }

export interface SnapResult { pt: Vec; kind: string | null }

export interface ViewCtx {
  doc: DocStore;
  sel: Selection;
  cam: Cam;
  unit(): string;
  toWorld(sx: number, sy: number): Vec;
  toScreen(w: Vec): Vec;
  tol(px: number): number;
  snap(w: Vec, exclude?: number[]): SnapResult;
  setSnapMark(m: SnapResult | null): void;
  redraw(): void;
  cursor(css: string): void;
  exact(commandId: string, initial: Record<string, unknown>): Promise<Record<string, unknown> | null>;
  say(text: string): void;
  setHover(h: Hit | null): void;
  frame(ids?: number[]): void;
  selected(): void;
}

export interface Tool {
  id: string;
  label: string;
  key: string;
  icon: string;
  down(p: Ptr): void;
  move(p: Ptr): void;
  up(p: Ptr): void;
  dbl?(p: Ptr): void;
  keydown?(e: KeyboardEvent): boolean;
  exact(): void;
  cancel(): void;
  busy(): boolean;
  draw(g: CanvasRenderingContext2D, c: Colors): void;
  leave?(): void;
}
