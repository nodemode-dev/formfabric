export type Gesture = 'box' | 'lasso' | 'line';

export interface Drawn {
  kind: Gesture;
  pts: number[];
}

export class SelectLayer {
  readonly canvas: HTMLCanvasElement;
  private readonly ctx: CanvasRenderingContext2D;
  private kind: Gesture | null = null;
  private pts: number[] = [];
  private dpr = 1;
  private accent = '';

  constructor(pane: HTMLElement) {
    this.canvas = document.createElement('canvas');
    this.canvas.className = 'ff3-select';
    this.canvas.style.cssText = 'position:absolute;left:0;top:0;pointer-events:none';
    this.ctx = this.canvas.getContext('2d') as CanvasRenderingContext2D;
    pane.appendChild(this.canvas);
  }

  get busy(): boolean { return this.kind !== null; }

  resize(w: number, h: number, dpr: number): void {
    const bw = Math.round(w * dpr);
    const bh = Math.round(h * dpr);
    if (this.canvas.width !== bw) this.canvas.width = bw;
    if (this.canvas.height !== bh) this.canvas.height = bh;
    this.canvas.style.width = `${w}px`;
    this.canvas.style.height = `${h}px`;
    this.dpr = dpr;
    this.draw();
  }

  begin(kind: Gesture, x: number, y: number, accent: string): void {
    this.kind = kind;
    this.accent = accent;
    this.pts = kind === 'lasso' ? [x, y] : [x, y, x, y];
    this.draw();
  }

  move(x: number, y: number): void {
    const p = this.pts;
    if (!this.kind) return;
    if (this.kind === 'lasso') {
      const n = p.length;
      if (Math.hypot(x - p[n - 2], y - p[n - 1]) < 3) return;
      p.push(x, y);
    } else {
      p[2] = x;
      p[3] = y;
    }
    this.draw();
  }

  finish(): Drawn | null {
    const kind = this.kind;
    const pts = this.pts;
    this.kind = null;
    this.pts = [];
    this.draw();
    return kind ? { kind, pts } : null;
  }

  private draw(): void {
    const c = this.ctx;
    c.clearRect(0, 0, this.canvas.width, this.canvas.height);
    const p = this.pts;
    if (!this.kind || p.length < 4) return;
    const d = this.dpr;
    c.lineWidth = 1.5 * d;
    c.strokeStyle = this.accent;
    c.fillStyle = this.accent;
    c.setLineDash([6 * d, 4 * d]);
    c.beginPath();
    if (this.kind === 'box') {
      c.rect(Math.min(p[0], p[2]) * d, Math.min(p[1], p[3]) * d, Math.abs(p[2] - p[0]) * d, Math.abs(p[3] - p[1]) * d);
    } else {
      c.moveTo(p[0] * d, p[1] * d);
      for (let i = 2; i < p.length; i += 2) c.lineTo(p[i] * d, p[i + 1] * d);
      if (this.kind === 'lasso') c.closePath();
    }
    if (this.kind !== 'line') {
      c.globalAlpha = 0.12;
      c.fill();
      c.globalAlpha = 1;
    }
    c.stroke();
    c.setLineDash([]);
  }
}
