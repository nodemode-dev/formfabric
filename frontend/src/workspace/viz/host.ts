import type { OrbitCamera } from '../view3d/camera';
import type { Fx } from '../view3d/fx';
import type { Scene } from '../view3d/scene';
import type { VizJson } from './types';

export interface WorldHit {
  panel: number;
  vertex: number;
  point: [number, number, number];
}

export interface VizHost {
  pane: HTMLElement;
  bar: HTMLElement;
  canvas: HTMLCanvasElement;
  camera: OrbitCamera;
  scene: Scene;
  fx: Fx;
  gl(): WebGL2RenderingContext | null;
  viz(): VizJson;
  units(): string;
  redraw(): void;
  paintNow(): void;
  snapshot(width: number, height: number, transparent: boolean): HTMLCanvasElement | null;
  maxSize(): number;
  clearScalars(): void;
  setAutoFrame(on: boolean): void;
}
