import { serviceUrl } from '../../engine/service';
import type { LightJson, StageJson } from '../viz/types';
import { MAX_LIGHTS, SHADOW_GLSL, type Fx } from './fx';
import { invertInto, type Frame } from './frame';
import { lookAt, multiply, normalize, ortho, perspective, type Vec3 } from './math';
import type { Scene } from './scene';

const SHADOW_SIZE = 2048;
const SH_W = 64;
const SH_H = 32;
const KIND_CODE: Record<string, number> = { directional: 0, point: 1, spot: 2, area: 3 };

const DEPTH_VS = `#version 300 es
layout(location = 0) in vec3 aPos;
uniform mat4 uLP;
uniform mat4 uModel;
void main() {
  gl_Position = uLP * (uModel * vec4(aPos, 1.0));
}`;

const DEPTH_FS = `#version 300 es
precision highp float;
void main() {}`;

const SKY_VS = `#version 300 es
uniform mat4 uInv;
uniform vec3 uEye;
out vec3 vDir;
void main() {
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2)) * 2.0 - 1.0;
  vec4 f = uInv * vec4(p, 1.0, 1.0);
  vDir = f.xyz / f.w - uEye;
  gl_Position = vec4(p, 0.9999, 1.0);
}`;

const SKY_FS = `#version 300 es
precision highp float;
in vec3 vDir;
uniform sampler2D uSky;
uniform float uExposure;
uniform float uRot;
out vec4 outColor;
void main() {
  vec3 d = normalize(vDir);
  float c = cos(uRot);
  float s = sin(uRot);
  vec3 m = vec3(c * d.x + s * d.z, d.y, -s * d.x + c * d.z);
  vec2 uv = vec2(atan(m.x, m.z) / 6.2831853 + 0.5, acos(clamp(m.y, -1.0, 1.0)) / 3.1415927);
  vec3 col = texture(uSky, uv).rgb * uExposure;
  col = col / (1.0 + col * 0.18);
  outColor = vec4(pow(col, vec3(1.0 / 2.2)), 1.0);
}`;

const GROUND_VS = `#version 300 es
layout(location = 0) in vec2 aPos;
uniform mat4 uVP;
uniform vec3 uCenter;
uniform float uSize;
out vec3 vW;
void main() {
  vec3 w = vec3(uCenter.x + aPos.x * uSize, uCenter.y, uCenter.z + aPos.y * uSize);
  vW = w;
  gl_Position = uVP * vec4(w, 1.0);
}`;

const GROUND_FS = `#version 300 es
precision highp float;
precision highp sampler2DShadow;
in vec3 vW;
uniform vec3 uCenter;
uniform float uSize;
uniform vec3 uFloor;
uniform float uHasShadow;
${SHADOW_GLSL}
out vec4 outColor;
void main() {
  float sh = uHasShadow > 0.5 ? shadowAt(vW, vec3(0.0, 1.0, 0.0)) : 1.0;
  float fade = 1.0 - smoothstep(0.1, 0.5, distance(vW.xz, uCenter.xz) / uSize);
  vec3 col = uFloor * (1.0 - 0.7 * (1.0 - sh));
  outColor = vec4(pow(col, vec3(1.0 / 2.2)), fade * (0.5 + 0.35 * (1.0 - sh)));
}`;

function compile(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader {
  const sh = gl.createShader(type) as WebGLShader;
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(sh) ?? 'shader');
  return sh;
}

function link(gl: WebGL2RenderingContext, vs: string, fs: string): WebGLProgram {
  const p = gl.createProgram() as WebGLProgram;
  gl.attachShader(p, compile(gl, gl.VERTEX_SHADER, vs));
  gl.attachShader(p, compile(gl, gl.FRAGMENT_SHADER, fs));
  gl.linkProgram(p);
  if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p) ?? 'link');
  return p;
}

export function parseBackground(v: unknown): [number, number, number] | null {
  let s: unknown = v;
  if (s && typeof s === 'object' && !Array.isArray(s)) s = (s as { color?: unknown }).color;
  if (typeof s === 'string') {
    const m = /^#?([0-9a-f]{6})$/i.exec(s.trim());
    if (!m) return null;
    const n = parseInt(m[1], 16);
    return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
  }
  if (Array.isArray(s) && s.length >= 3 && s.every((x) => typeof x === 'number')) {
    const a = s as number[];
    const k = a.some((x) => x > 1) ? 255 : 1;
    return [a[0] / k, a[1] / k, a[2] / k];
  }
  return null;
}

export class StageGL {
  stage: StageJson | null = null;
  hdriState: 'none' | 'loading' | 'ready' | 'failed' = 'none';
  onLoaded: (() => void) | null = null;
  private readonly depthProg: WebGLProgram;
  private readonly dLP: WebGLUniformLocation | null;
  private readonly dModel: WebGLUniformLocation | null;
  private readonly skyProg: WebGLProgram;
  private readonly sInv: WebGLUniformLocation | null;
  private readonly sEye: WebGLUniformLocation | null;
  private readonly sSky: WebGLUniformLocation | null;
  private readonly sExp: WebGLUniformLocation | null;
  private readonly sRot: WebGLUniformLocation | null;
  private readonly groundProg: WebGLProgram;
  private readonly gu: Record<string, WebGLUniformLocation | null> = {};
  private readonly groundVao: WebGLVertexArrayObject | null;
  private readonly emptyVao: WebGLVertexArrayObject | null;
  private readonly fbo: WebGLFramebuffer | null;
  private readonly depthTex: WebGLTexture | null;
  private sky: WebGLTexture | null = null;
  private skyUrl = '';
  private shadowKey = '';
  private readonly tmp = new Float32Array(16);
  private readonly inv = new Float32Array(16);
  private readonly center: Vec3 = [0, 900, 0];
  private radius = 1000;
  private minY = 0;
  private readonly lo = new Float32Array(3);
  private readonly hi = new Float32Array(3);
  private readonly lightMat = new Float32Array(16);
  floor = new Float32Array([0.05, 0.05, 0.05]);

  constructor(private readonly gl: WebGL2RenderingContext) {
    this.depthProg = link(gl, DEPTH_VS, DEPTH_FS);
    this.dLP = gl.getUniformLocation(this.depthProg, 'uLP');
    this.dModel = gl.getUniformLocation(this.depthProg, 'uModel');
    this.skyProg = link(gl, SKY_VS, SKY_FS);
    this.sInv = gl.getUniformLocation(this.skyProg, 'uInv');
    this.sEye = gl.getUniformLocation(this.skyProg, 'uEye');
    this.sSky = gl.getUniformLocation(this.skyProg, 'uSky');
    this.sExp = gl.getUniformLocation(this.skyProg, 'uExposure');
    this.sRot = gl.getUniformLocation(this.skyProg, 'uRot');
    this.groundProg = link(gl, GROUND_VS, GROUND_FS);
    for (const n of ['uVP', 'uCenter', 'uSize', 'uFloor', 'uHasShadow', 'uShadow', 'uShadowM', 'uShadowTexel']) this.gu[n] = gl.getUniformLocation(this.groundProg, n);
    this.groundVao = gl.createVertexArray();
    gl.bindVertexArray(this.groundVao);
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
    this.emptyVao = gl.createVertexArray();
    this.depthTex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, this.depthTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.DEPTH_COMPONENT24, SHADOW_SIZE, SHADOW_SIZE, 0, gl.DEPTH_COMPONENT, gl.UNSIGNED_INT, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_MODE, gl.COMPARE_REF_TO_TEXTURE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_FUNC, gl.LEQUAL);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.bindTexture(gl.TEXTURE_2D, null);
    this.fbo = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, this.fbo);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, this.depthTex, 0);
    gl.drawBuffers([gl.NONE]);
    gl.readBuffer(gl.NONE);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  }

  get active(): boolean {
    const s = this.stage;
    return !!s && (s.lights.length > 0 || s.hdri !== null || s.ground);
  }

  get hasSky(): boolean {
    return this.hdriState === 'ready' && this.sky !== null;
  }

  get groundOn(): boolean {
    return !!this.stage?.ground;
  }

  setStage(s: StageJson): void {
    this.stage = s;
    const url = s.hdri ?? '';
    if (url === this.skyUrl) return;
    this.skyUrl = url;
    this.shadowKey = '';
    if (!url) {
      this.hdriState = 'none';
      return;
    }
    this.hdriState = 'loading';
    void this.loadSky(url);
  }

  private async loadSky(path: string): Promise<void> {
    const gl = this.gl;
    try {
      const direct = /^(https?:|blob:|data:)/.test(path);
      const res = await fetch(direct ? path : serviceUrl(`/api/files/image?path=${encodeURIComponent(path)}`), { cache: 'no-store' });
      if (!res.ok) throw new Error(String(res.status));
      const bmp = await createImageBitmap(await res.blob(), { premultiplyAlpha: 'none', colorSpaceConversion: 'none' });
      if (path !== this.skyUrl) {
        bmp.close();
        return;
      }
      const tex = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, tex);
      gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.SRGB8_ALPHA8, gl.RGBA, gl.UNSIGNED_BYTE, bmp);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.bindTexture(gl.TEXTURE_2D, null);
      if (this.sky) gl.deleteTexture(this.sky);
      this.sky = tex;
      this.irradiance(bmp);
      bmp.close();
      this.hdriState = 'ready';
    } catch {
      this.hdriState = 'failed';
    }
    this.onLoaded?.();
  }

  private readonly sh = new Float32Array(27);

  private irradiance(bmp: ImageBitmap): void {
    const c = document.createElement('canvas');
    c.width = SH_W;
    c.height = SH_H;
    const ctx = c.getContext('2d', { willReadFrequently: true }) as CanvasRenderingContext2D;
    ctx.drawImage(bmp, 0, 0, SH_W, SH_H);
    const px = ctx.getImageData(0, 0, SH_W, SH_H).data;
    const acc = new Float64Array(27);
    for (let j = 0; j < SH_H; j++) {
      const theta = ((j + 0.5) / SH_H) * Math.PI;
      const st = Math.sin(theta);
      const dw = ((2 * Math.PI) / SH_W) * (Math.PI / SH_H) * st;
      for (let i = 0; i < SH_W; i++) {
        const phi = ((i + 0.5) / SH_W - 0.5) * 2 * Math.PI;
        const x = st * Math.sin(phi);
        const z = st * Math.cos(phi);
        const y = Math.cos(theta);
        const k = (j * SH_W + i) * 4;
        const r = Math.pow(px[k] / 255, 2.2) * dw;
        const g = Math.pow(px[k + 1] / 255, 2.2) * dw;
        const b = Math.pow(px[k + 2] / 255, 2.2) * dw;
        const basis = [0.282095, 0.488603 * y, 0.488603 * z, 0.488603 * x, 1.092548 * x * y, 1.092548 * y * z, 0.315392 * (3 * z * z - 1), 1.092548 * x * z, 0.546274 * (x * x - y * y)];
        for (let n = 0; n < 9; n++) {
          acc[n * 3] += r * basis[n];
          acc[n * 3 + 1] += g * basis[n];
          acc[n * 3 + 2] += b * basis[n];
        }
      }
    }
    const band = [1, 2 / 3, 2 / 3, 2 / 3, 0.25, 0.25, 0.25, 0.25, 0.25];
    for (let n = 0; n < 9; n++) for (let k = 0; k < 3; k++) this.sh[n * 3 + k] = acc[n * 3 + k] * band[n];
  }

  sync(scene: Scene, fx: Fx): void {
    const st = fx.stage;
    const s = this.stage;
    if (!s || !this.active) {
      st.on = false;
      return;
    }
    const lights = s.lights.slice(0, MAX_LIGHTS);
    st.on = true;
    st.count = lights.length;
    st.exposure = Math.pow(2, Number.isFinite(s.exposure) ? s.exposure : 0);
    st.rotation = (s.hdri_rotation * Math.PI) / 180;
    st.hasSH = this.hdriState === 'ready';
    if (st.hasSH) st.sh.set(this.sh);
    const any = scene.bounds(null, this.lo, this.hi);
    if (any) {
      this.center[0] = (this.lo[0] + this.hi[0]) / 2;
      this.center[1] = (this.lo[1] + this.hi[1]) / 2;
      this.center[2] = (this.lo[2] + this.hi[2]) / 2;
      this.radius = Math.max(50, 0.5 * Math.hypot(this.hi[0] - this.lo[0], this.hi[1] - this.lo[1], this.hi[2] - this.lo[2]));
      this.minY = this.lo[1];
    }
    st.ambient[0] = st.ambient[1] = st.ambient[2] = lights.length ? 0.1 : 0.5;
    let key = -1;
    for (let i = 0; i < lights.length; i++) {
      const l = lights[i];
      st.pos[i * 4] = l.position[0];
      st.pos[i * 4 + 1] = l.position[1];
      st.pos[i * 4 + 2] = l.position[2];
      st.pos[i * 4 + 3] = KIND_CODE[l.kind] ?? 0;
      const d = lightDir(l, this.center);
      st.dir[i * 3] = d[0];
      st.dir[i * 3 + 1] = d[1];
      st.dir[i * 3 + 2] = d[2];
      const k = Math.max(0, l.intensity);
      st.col[i * 3] = l.color[0] * k;
      st.col[i * 3 + 1] = l.color[1] * k;
      st.col[i * 3 + 2] = l.color[2] * k;
      if (key < 0 && l.shadows && any) key = i;
    }
    st.shadowIdx = key;
    st.shadowTexel = 1 / SHADOW_SIZE;
    st.shadowTex = this.depthTex;
    if (key < 0) return;
    this.lightMatrix(lights[key], this.lightMat);
    st.shadowMat.set(this.lightMat);
    const stamp = `${scene.version}|${Array.from(this.lightMat).join(',')}`;
    if (stamp === this.shadowKey) return;
    this.shadowKey = stamp;
    this.renderShadow(scene);
  }

  private lightMatrix(l: LightJson, out: Float32Array): void {
    const c = this.center;
    const r = this.radius * 1.08;
    const d = lightDir(l, c);
    const up: Vec3 = Math.abs(d[1]) > 0.98 ? [0, 0, 1] : [0, 1, 0];
    let view: Float32Array;
    let proj: Float32Array;
    if (l.kind === 'directional') {
      const eye: Vec3 = [c[0] - d[0] * r * 3, c[1] - d[1] * r * 3, c[2] - d[2] * r * 3];
      view = lookAt(eye, c, up);
      proj = ortho(r, 1, 0.1, r * 6);
    } else {
      const eye: Vec3 = [l.position[0], l.position[1], l.position[2]];
      const dist = Math.max(Math.hypot(c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]), r * 1.2);
      view = lookAt(eye, c, Math.abs(c[1] - eye[1]) > dist * 0.98 ? [0, 0, 1] : [0, 1, 0]);
      const fov = Math.min(2.4, 2 * Math.asin(Math.min(0.95, (r * 1.1) / dist)));
      proj = perspective(fov, 1, Math.max(dist - r * 1.2, dist * 0.05), dist + r * 1.2);
    }
    out.set(multiply(proj, view));
  }

  private renderShadow(scene: Scene): void {
    const gl = this.gl;
    const prev = gl.getParameter(gl.VIEWPORT) as Int32Array;
    const prevFbo = gl.getParameter(gl.FRAMEBUFFER_BINDING) as WebGLFramebuffer | null;
    gl.bindFramebuffer(gl.FRAMEBUFFER, this.fbo);
    gl.viewport(0, 0, SHADOW_SIZE, SHADOW_SIZE);
    gl.depthMask(true);
    gl.clearDepth(1);
    gl.clear(gl.DEPTH_BUFFER_BIT);
    gl.colorMask(false, false, false, false);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.disable(gl.CULL_FACE);
    gl.enable(gl.POLYGON_OFFSET_FILL);
    gl.polygonOffset(2, 4);
    gl.useProgram(this.depthProg);
    gl.uniformMatrix4fv(this.dLP, false, this.lightMat);
    for (const b of scene.list) {
      if (!b.ready || !b.vao) continue;
      gl.uniformMatrix4fv(this.dModel, false, b.model);
      gl.bindVertexArray(b.vao);
      gl.drawElements(gl.TRIANGLES, b.tcount * 3, gl.UNSIGNED_INT, 0);
    }
    gl.bindVertexArray(null);
    gl.disable(gl.POLYGON_OFFSET_FILL);
    gl.colorMask(true, true, true, true);
    gl.disable(gl.DEPTH_TEST);
    gl.bindFramebuffer(gl.FRAMEBUFFER, prevFbo);
    gl.viewport(prev[0], prev[1], prev[2], prev[3]);
  }

  drawSky(f: Frame, fx: Fx): void {
    if (!this.hasSky || !this.sky) return;
    const gl = this.gl;
    if (!invertInto(this.inv, f.vp, this.tmp)) return;
    gl.useProgram(this.skyProg);
    gl.uniformMatrix4fv(this.sInv, false, this.inv);
    gl.uniform3f(this.sEye, f.eye[0], f.eye[1], f.eye[2]);
    gl.uniform1f(this.sExp, fx.stage.exposure);
    gl.uniform1f(this.sRot, fx.stage.rotation);
    gl.uniform1i(this.sSky, 3);
    gl.activeTexture(gl.TEXTURE3);
    gl.bindTexture(gl.TEXTURE_2D, this.sky);
    gl.disable(gl.DEPTH_TEST);
    gl.disable(gl.BLEND);
    gl.bindVertexArray(this.emptyVao);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    gl.bindVertexArray(null);
    gl.bindTexture(gl.TEXTURE_2D, null);
    gl.activeTexture(gl.TEXTURE0);
  }

  drawGround(f: Frame, fx: Fx): void {
    if (!this.groundOn) return;
    const gl = this.gl;
    const u = this.gu;
    gl.useProgram(this.groundProg);
    gl.uniformMatrix4fv(u.uVP, false, f.vp);
    gl.uniform3f(u.uCenter, this.center[0], this.minY - 1, this.center[2]);
    gl.uniform1f(u.uSize, this.radius * 14);
    gl.uniform3f(u.uFloor, this.floor[0], this.floor[1], this.floor[2]);
    const shadow = fx.stage.shadowIdx >= 0;
    gl.uniform1f(u.uHasShadow, shadow ? 1 : 0);
    gl.uniform1i(u.uShadow, 2);
    gl.uniformMatrix4fv(u.uShadowM, false, fx.stage.shadowMat);
    gl.uniform1f(u.uShadowTexel, fx.stage.shadowTexel);
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, shadow ? this.depthTex : null);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(false);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    gl.bindVertexArray(this.groundVao);
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    gl.bindVertexArray(null);
    gl.depthMask(true);
    gl.disable(gl.BLEND);
    gl.disable(gl.DEPTH_TEST);
    gl.bindTexture(gl.TEXTURE_2D, null);
    gl.activeTexture(gl.TEXTURE0);
  }
}

function lightDir(l: LightJson, center: Vec3): Vec3 {
  if (l.kind === 'directional' || l.kind === 'spot') {
    const d = l.direction;
    if (Math.hypot(d[0], d[1], d[2]) > 1e-6) return normalize([d[0], d[1], d[2]]);
  }
  return normalize([center[0] - l.position[0], center[1] - l.position[1], center[2] - l.position[2]]);
}
