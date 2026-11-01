import { hueColor, linear, type PanelLook } from './look';

export type MeshSource = 'local' | 'mesh' | 'stream' | 'feed';

const IDENTITY = new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);

export class PanelBuf {
  vcount = 0;
  tcount = 0;
  pos = new Float32Array(0);
  nrm = new Float32Array(0);
  uv = new Float32Array(0);
  idx = new Uint32Array(0);
  edges = new Uint32Array(0);
  ecount = 0;
  local: Float32Array | null = null;
  source: MeshSource = 'local';
  kind: 'panel' | 'body' = 'panel';
  shapeKey = '';
  placeKey = '';
  seq = 0;
  ready = false;
  readonly min = new Float32Array(3);
  readonly max = new Float32Array(3);
  readonly model = new Float32Array(IDENTITY);
  holdModel = false;
  moved = false;
  stateKind = 0;
  readonly front = new Float32Array(3);
  readonly back = new Float32Array(3);
  readonly hue = new Float32Array(3);
  rough = 0.85;
  sheen = 0.2;
  map: string | null = null;
  readonly mapXf = new Float32Array(4);
  readonly mapOff = new Float32Array(2);
  pending: { indices: Uint32Array; vcount: number } | null = null;
  topoDirty = false;
  posDirty = false;
  uvDirty = false;
  edgesDirty = true;
  vao: WebGLVertexArrayObject | null = null;
  vaoWire: WebGLVertexArrayObject | null = null;
  bPos: WebGLBuffer | null = null;
  bNrm: WebGLBuffer | null = null;
  bUv: WebGLBuffer | null = null;
  bIdx: WebGLBuffer | null = null;
  bEdge: WebGLBuffer | null = null;
  gpuV = -1;
  gpuE = -1;
  scal = new Float32Array(0);
  scalDirty = false;
  scalOn = false;
  bScal: WebGLBuffer | null = null;

  constructor(readonly id: number) {
    const h = hueColor(id);
    this.hue.set(linear(h));
    this.min.fill(0);
    this.max.fill(0);
  }

  setLook(l: PanelLook): void {
    this.front.set(linear(l.front));
    this.back.set(linear(l.back));
    this.rough = l.rough;
    this.sheen = l.sheen;
    this.map = l.map;
    const k = 1 / l.mapScale;
    this.mapXf[0] = Math.cos(l.mapRot) * k;
    this.mapXf[1] = Math.sin(l.mapRot) * k;
    this.mapOff[0] = l.mapOff[0];
    this.mapOff[1] = l.mapOff[1];
  }

  resetModel(): void {
    this.model.set(IDENTITY);
    this.moved = false;
    this.holdModel = false;
  }
}

export class Scene {
  readonly list: PanelBuf[] = [];
  private readonly byId = new Map<number, PanelBuf>();
  private readonly dead: PanelBuf[] = [];
  triangles = 0;
  vertices = 0;
  panels = 0;
  version = 0;
  private statsDirty = true;

  get(id: number): PanelBuf | undefined {
    return this.byId.get(id);
  }

  ensure(id: number): PanelBuf {
    let b = this.byId.get(id);
    if (!b) {
      b = new PanelBuf(id);
      this.byId.set(id, b);
      this.list.push(b);
      this.version++;
    }
    return b;
  }

  remove(id: number): PanelBuf | undefined {
    const b = this.byId.get(id);
    if (!b) return undefined;
    this.byId.delete(id);
    this.list.splice(this.list.indexOf(b), 1);
    this.dead.push(b);
    this.statsDirty = true;
    this.version++;
    return b;
  }

  setTopology(b: PanelBuf, indices: Uint32Array, vcount: number): void {
    if (b.idx.length !== indices.length) b.idx = new Uint32Array(indices.length);
    b.idx.set(indices);
    b.tcount = indices.length / 3;
    if (b.pos.length !== vcount * 3) {
      b.pos = new Float32Array(vcount * 3);
      b.nrm = new Float32Array(vcount * 3);
    }
    if (b.uv.length !== vcount * 2) {
      b.uv = new Float32Array(vcount * 2);
      b.uvDirty = true;
    }
    b.vcount = vcount;
    b.topoDirty = true;
    b.edgesDirty = true;
    b.ready = false;
    this.statsDirty = true;
  }

  setScalars(b: PanelBuf, values: Float32Array | null): void {
    if (!values || values.length !== b.vcount) {
      if (b.scalOn) {
        b.scalOn = false;
        b.scalDirty = true;
      }
      return;
    }
    if (b.scal.length !== values.length) b.scal = new Float32Array(values.length);
    b.scal.set(values);
    b.scalOn = true;
    b.scalDirty = true;
    this.version++;
  }

  setUv(b: PanelBuf, local: Float32Array | null): void {
    if (local && local.length === b.vcount * 2) b.uv.set(local);
    else b.uv.fill(0);
    b.uvDirty = true;
  }

  setPositions(b: PanelBuf, src: Float32Array): void {
    if (src.length !== b.vcount * 3) return;
    b.pos.set(src);
    this.afterPositions(b);
  }

  afterPositions(b: PanelBuf): void {
    const p = b.pos;
    let x0 = Infinity, y0 = Infinity, z0 = Infinity, x1 = -Infinity, y1 = -Infinity, z1 = -Infinity;
    for (let i = 0; i < p.length; i += 3) {
      const x = p[i], y = p[i + 1], z = p[i + 2];
      if (x < x0) x0 = x;
      if (x > x1) x1 = x;
      if (y < y0) y0 = y;
      if (y > y1) y1 = y;
      if (z < z0) z0 = z;
      if (z > z1) z1 = z;
    }
    if (x0 === Infinity) { x0 = y0 = z0 = x1 = y1 = z1 = 0; }
    b.min[0] = x0; b.min[1] = y0; b.min[2] = z0;
    b.max[0] = x1; b.max[1] = y1; b.max[2] = z1;
    computeNormals(b);
    b.posDirty = true;
    b.ready = true;
    if (b.holdModel) b.resetModel();
    this.version++;
  }

  recount(): void {
    if (!this.statsDirty) return;
    let t = 0;
    let v = 0;
    let n = 0;
    for (const b of this.list) {
      if (b.kind === 'panel') n++;
      if (!b.ready) continue;
      t += b.tcount;
      v += b.vcount;
    }
    this.triangles = t;
    this.vertices = v;
    this.panels = n;
    this.statsDirty = false;
  }

  touch(): void {
    this.statsDirty = true;
    this.version++;
  }

  bounds(ids: Set<number> | null, min: Float32Array, max: Float32Array): boolean {
    let any = false;
    for (const b of this.list) {
      if (!b.ready || (ids && !ids.has(b.id))) continue;
      for (let k = 0; k < 3; k++) {
        const lo = b.min[k];
        const hi = b.max[k];
        if (!any || lo < min[k]) min[k] = lo;
        if (!any || hi > max[k]) max[k] = hi;
      }
      any = true;
    }
    return any;
  }

  upload(gl: WebGL2RenderingContext, wire: boolean): void {
    while (this.dead.length) this.dispose(gl, this.dead.pop() as PanelBuf);
    for (const b of this.list) {
      if (!b.vao) create(gl, b);
      if (b.topoDirty) {
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, b.bIdx);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, b.idx, gl.STATIC_DRAW);
        b.topoDirty = false;
        b.gpuV = -1;
      }
      if (b.posDirty || b.gpuV !== b.vcount) {
        const resize = b.gpuV !== b.vcount;
        gl.bindBuffer(gl.ARRAY_BUFFER, b.bPos);
        if (resize) gl.bufferData(gl.ARRAY_BUFFER, b.pos, gl.DYNAMIC_DRAW);
        else gl.bufferSubData(gl.ARRAY_BUFFER, 0, b.pos);
        gl.bindBuffer(gl.ARRAY_BUFFER, b.bNrm);
        if (resize) gl.bufferData(gl.ARRAY_BUFFER, b.nrm, gl.DYNAMIC_DRAW);
        else gl.bufferSubData(gl.ARRAY_BUFFER, 0, b.nrm);
        if (resize) {
          gl.bindBuffer(gl.ARRAY_BUFFER, b.bUv);
          gl.bufferData(gl.ARRAY_BUFFER, b.uv, gl.STATIC_DRAW);
          b.uvDirty = false;
        }
        b.gpuV = b.vcount;
        b.posDirty = false;
      }
      if (b.uvDirty) {
        gl.bindBuffer(gl.ARRAY_BUFFER, b.bUv);
        gl.bufferData(gl.ARRAY_BUFFER, b.uv, gl.STATIC_DRAW);
        b.uvDirty = false;
      }
      if (b.scalDirty || (b.scalOn && b.scal.length !== b.vcount)) {
        if (b.scalOn && b.scal.length === b.vcount) {
          gl.bindVertexArray(b.vao);
          gl.bindBuffer(gl.ARRAY_BUFFER, b.bScal);
          gl.bufferData(gl.ARRAY_BUFFER, b.scal, gl.DYNAMIC_DRAW);
          gl.enableVertexAttribArray(3);
          gl.vertexAttribPointer(3, 1, gl.FLOAT, false, 0, 0);
        } else {
          b.scalOn = false;
          gl.bindVertexArray(b.vao);
          gl.disableVertexAttribArray(3);
          gl.vertexAttrib1f(3, 0);
        }
        gl.bindVertexArray(null);
        b.scalDirty = false;
      }
      if (wire && b.edgesDirty && b.kind === 'panel') {
        buildEdges(b);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, b.bEdge);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, b.edges.subarray(0, b.ecount * 2), gl.STATIC_DRAW);
        b.edgesDirty = false;
      }
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  dispose(gl: WebGL2RenderingContext, b: PanelBuf): void {
    gl.deleteVertexArray(b.vao);
    gl.deleteVertexArray(b.vaoWire);
    gl.deleteBuffer(b.bPos);
    gl.deleteBuffer(b.bNrm);
    gl.deleteBuffer(b.bUv);
    gl.deleteBuffer(b.bIdx);
    gl.deleteBuffer(b.bEdge);
    gl.deleteBuffer(b.bScal);
  }
}

function create(gl: WebGL2RenderingContext, b: PanelBuf): void {
  b.bPos = gl.createBuffer();
  b.bNrm = gl.createBuffer();
  b.bUv = gl.createBuffer();
  b.bIdx = gl.createBuffer();
  b.bEdge = gl.createBuffer();
  b.bScal = gl.createBuffer();
  b.vao = gl.createVertexArray();
  gl.bindVertexArray(b.vao);
  gl.bindBuffer(gl.ARRAY_BUFFER, b.bPos);
  gl.enableVertexAttribArray(0);
  gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ARRAY_BUFFER, b.bNrm);
  gl.enableVertexAttribArray(1);
  gl.vertexAttribPointer(1, 3, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ARRAY_BUFFER, b.bUv);
  gl.enableVertexAttribArray(2);
  gl.vertexAttribPointer(2, 2, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, b.bIdx);
  b.vaoWire = gl.createVertexArray();
  gl.bindVertexArray(b.vaoWire);
  gl.bindBuffer(gl.ARRAY_BUFFER, b.bPos);
  gl.enableVertexAttribArray(0);
  gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, b.bEdge);
  gl.bindVertexArray(null);
}

export function computeNormals(b: PanelBuf): void {
  const p = b.pos;
  const n = b.nrm;
  const idx = b.idx;
  n.fill(0);
  for (let t = 0; t < idx.length; t += 3) {
    const a = idx[t] * 3;
    const bb = idx[t + 1] * 3;
    const c = idx[t + 2] * 3;
    const e1x = p[bb] - p[a], e1y = p[bb + 1] - p[a + 1], e1z = p[bb + 2] - p[a + 2];
    const e2x = p[c] - p[a], e2y = p[c + 1] - p[a + 1], e2z = p[c + 2] - p[a + 2];
    const nx = e1y * e2z - e1z * e2y;
    const ny = e1z * e2x - e1x * e2z;
    const nz = e1x * e2y - e1y * e2x;
    n[a] += nx; n[a + 1] += ny; n[a + 2] += nz;
    n[bb] += nx; n[bb + 1] += ny; n[bb + 2] += nz;
    n[c] += nx; n[c + 1] += ny; n[c + 2] += nz;
  }
  for (let i = 0; i < n.length; i += 3) {
    const x = n[i], y = n[i + 1], z = n[i + 2];
    const l = Math.sqrt(x * x + y * y + z * z);
    if (l > 1e-12) {
      const k = 1 / l;
      n[i] = x * k; n[i + 1] = y * k; n[i + 2] = z * k;
    } else {
      n[i] = 0; n[i + 1] = 0; n[i + 2] = 1;
    }
  }
}

function buildEdges(b: PanelBuf): void {
  const idx = b.idx;
  const nv = Math.max(b.vcount, 1);
  const keys = new Float64Array(idx.length);
  let k = 0;
  for (let t = 0; t < idx.length; t += 3) {
    for (let e = 0; e < 3; e++) {
      const a = idx[t + e];
      const c = idx[t + ((e + 1) % 3)];
      keys[k++] = a < c ? a * nv + c : c * nv + a;
    }
  }
  keys.sort();
  if (b.edges.length < keys.length * 2) b.edges = new Uint32Array(keys.length * 2);
  let out = 0;
  let last = -1;
  for (let i = 0; i < keys.length; i++) {
    const key = keys[i];
    if (key === last) continue;
    last = key;
    const lo = Math.floor(key / nv);
    b.edges[out++] = lo;
    b.edges[out++] = key - lo * nv;
  }
  b.ecount = out / 2;
}
