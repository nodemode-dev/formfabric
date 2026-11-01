export interface SeamLine {
  pa: number;
  va: number;
  pb: number;
  vb: number;
}

export interface PinMark {
  id: number;
  panel: number;
  vertex: number;
}

export type Tool = 'none' | 'pinch' | 'box' | 'lasso' | 'fold' | 'tape';

export interface PanelStateJson {
  frozen?: boolean;
  disabled?: boolean;
  stiffen?: number;
  hold_shape?: boolean;
  shrink_weft?: number;
  shrink_warp?: number;
}

export interface WindJson {
  enabled: boolean;
  kind: string;
  origin: [number, number, number];
  direction: [number, number, number];
  strength: number;
  decay: number;
  turbulence: number;
}

export interface TapeMark {
  id: number;
  panel: number;
  vertex: number;
  point: [number, number, number];
}

export interface ToolOpts {
  radius: number;
  falloff: string;
  power: number;
  distance: string;
  angle: number;
  side: string;
  length: number;
}

export interface DrapeHost {
  redraw(): void;
  setSeams(lines: SeamLine[]): void;
  setPins(pins: PinMark[]): void;
  pinAt(panel: number, vertex: number): number;
  garment(): unknown;
  active(): boolean;
  selected(): number[];
  target(): [number, number, number];
}

export interface SimState {
  running: boolean;
  frame: number;
  vertices: number;
  ms: number;
  preset: string;
  panels: number;
}
