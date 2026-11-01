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

export interface DrapeHost {
  redraw(): void;
  setSeams(lines: SeamLine[]): void;
  setPins(pins: PinMark[]): void;
  pinAt(panel: number, vertex: number): number;
  garment(): unknown;
}

export interface SimState {
  running: boolean;
  frame: number;
  vertices: number;
  ms: number;
  preset: string;
  panels: number;
}
