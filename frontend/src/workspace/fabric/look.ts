import { el, q, setClass, setHidden, setText, setValue } from '../../ui/dom';
import { icon } from '../../ui/icons';
import { t } from '../../ui/i18n';
import type { Ctx } from './ctx';
import { canPickImage, pickImage, previewUrl } from './files';
import { fromHex, KINDS, toHex, type FabricInfo, type MapKey, type Surface, type SurfaceKey } from './types';
import { colorField, flagBad, label, numField, pairField, selectField, titled, toggleField, unitField, repaint } from './widgets';

const MAPS: Array<[MapKey, string]> = [['base_color', 'Base color'], ['normal', 'Normal'], ['roughness', 'Roughness']];
const SURFACES: Array<[SurfaceKey, string]> = [['front', 'Front'], ['back', 'Back'], ['side', 'Side']];
const KIND_LABELS: Array<[string, string]> = KINDS.map((k) => [k, k.charAt(0).toUpperCase() + k.slice(1)]);

interface MapRow {
  el: HTMLElement;
  input: HTMLInputElement;
  img: HTMLImageElement;
  pick: HTMLButtonElement;
  clear: HTMLButtonElement;
  shown: string;
}

export class LookSection {
  private surf: SurfaceKey = 'front';
  private readonly tabs = new Map<SurfaceKey, HTMLButtonElement>();
  private readonly kind;
  private readonly own;
  private readonly color;
  private readonly rough;
  private readonly metal;
  private readonly sheen;
  private readonly opacity;
  private readonly scale;
  private readonly rot;
  private readonly offset;
  private readonly maps = new Map<MapKey, MapRow>();

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    this.kind = selectField('Kind', KIND_LABELS.map(([v, text]) => [v, text] as const), (v) => void this.send({ kind: v }));
    host.appendChild(this.kind.el);

    const bar = el('<div class="ff-fb-tabs" role="tablist"></div>');
    for (const [key, text] of SURFACES) {
      const b = el<HTMLButtonElement>('<button class="ff-fb-tab" type="button" role="tab"></button>');
      label(b, text);
      b.addEventListener('click', () => { this.surf = key; this.update(); b.blur(); });
      this.tabs.set(key, b);
      bar.appendChild(b);
    }
    host.appendChild(bar);

    this.own = toggleField('Separate surface', (on) => void this.send({ [this.surf]: on ? {} : null }));
    host.appendChild(this.own.el);

    this.color = colorField('Color', (hex) => void this.commitColor(hex));
    this.rough = unitField('Roughness', (v, i) => void this.surfaceSend({ roughness: v }, i));
    this.metal = unitField('Metalness', (v, i) => void this.surfaceSend({ metalness: v }, i));
    this.sheen = unitField('Sheen', (v, i) => void this.surfaceSend({ sheen: v }, i));
    this.opacity = unitField('Opacity', (v, i) => void this.surfaceSend({ opacity: v }, i));
    for (const f of [this.color, this.rough, this.metal, this.sheen, this.opacity]) host.appendChild(f.el);

    for (const [key, text] of MAPS) {
      const row = this.mapRow(key, text);
      this.maps.set(key, row);
      host.appendChild(row.el);
    }

    this.scale = pairField('Map scale', 'mm', '1', ([w, h], i) => void this.surfaceSend({ map_scale: [w, h] }, i));
    this.rot = numField('Map rotation', '°', '1', (v, i) => void this.surfaceSend({ map_rotation: v }, i));
    this.offset = pairField('Map offset', 'mm', '1', ([x, y], i) => void this.surfaceSend({ map_offset: [x, y] }, i));
    for (const f of [this.scale, this.rot, this.offset]) host.appendChild(f.el);
    repaint(() => this.update());
  }

  private mapRow(key: MapKey, text: string): MapRow {
    const root = el(`<div class="cv-field ff-fb-row ff-fb-map"><span></span><span class="ff-fb-thumb"><img alt="" hidden></span><input class="cv-input" type="text" spellcheck="false" autocomplete="off" disabled><button class="cv-icon-btn" type="button" data-k="pick">${icon('folderOpen', 16)}</button><button class="cv-icon-btn" type="button" data-k="clear">${icon('close', 14)}</button></div>`);
    label(q(root, 'span:first-child'), text);
    const input = q<HTMLInputElement>(root, 'input');
    const img = q<HTMLImageElement>(root, 'img');
    const pick = q<HTMLButtonElement>(root, '[data-k="pick"]');
    const clear = q<HTMLButtonElement>(root, '[data-k="clear"]');
    titled(pick, 'Choose file...');
    titled(clear, 'Clear');
    setHidden(pick, !canPickImage());
    const set = (path: string | null, field: HTMLInputElement | null): void => { void this.surfaceSend({ maps: { [key]: path } }, field); };
    input.addEventListener('change', () => { const v = input.value.trim(); set(v || null, input); });
    input.addEventListener('keydown', (e) => { if (e.key === 'Enter') input.blur(); });
    pick.addEventListener('click', () => { void pickImage().then((p) => { if (p) set(p, null); }); pick.blur(); });
    clear.addEventListener('click', () => { set(null, null); clear.blur(); });
    img.addEventListener('error', () => { img.hidden = true; });
    return { el: root, input, img, pick, clear, shown: '' };
  }

  private fabric(): FabricInfo | undefined {
    return this.ctx.selected();
  }

  private async send(look: Record<string, unknown>, input?: HTMLInputElement | null): Promise<void> {
    const f = this.fabric();
    if (!f) return;
    const r = await this.ctx.store.attempt('fabric.update', { id: f.id, look });
    if (r.ok) return;
    this.ctx.fail(r.message);
    if (input) flagBad(input, r.message, () => this.update());
    else this.update();
  }

  private surfaceSend(patch: Record<string, unknown>, input?: HTMLInputElement | null): Promise<void> {
    return this.send({ [this.surf]: patch }, input);
  }

  private async commitColor(hex: string): Promise<void> {
    const f = this.fabric();
    if (!f) return;
    const base = (f.look[this.surf] ?? f.look.front).color;
    const rgba = fromHex(hex, base[3]);
    const active = this.ctx.store.list.active_colorway;
    if (this.surf === 'front' && active !== null) {
      const r = await this.ctx.store.run('colorway.update', { id: active, fabric_colors: { [String(f.id)]: rgba } });
      if (r === undefined) this.update();
      return;
    }
    await this.surfaceSend({ color: rgba });
  }

  update(): void {
    const f = this.fabric();
    const list = this.ctx.store.list;
    for (const [key, b] of this.tabs) { setClass(b, 'is-on', key === this.surf); b.setAttribute('aria-selected', String(key === this.surf)); }
    this.kind.set(f?.look.kind ?? 'matte');
    this.kind.disable(!f);
    const own = this.surf === 'front' || (f !== undefined && f.look[this.surf] !== null);
    const s: Surface | undefined = f ? (f.look[this.surf] ?? f.look.front) : undefined;
    setClass(this.own.el, 'is-ghost', this.surf === 'front');
    this.own.set(own && this.surf !== 'front');
    if (this.own.input.disabled !== (!f || this.surf === 'front')) this.own.input.disabled = !f || this.surf === 'front';
    const off = !f || !own;
    const colorway = this.surf === 'front' && list.active_colorway !== null;
    setText(this.color.name, t(colorway ? 'Colorway color' : 'Color'));
    this.color.set(f && s ? toHex(colorway ? f.color : s.color) : '#000000');
    this.color.disable(off);
    this.rough.set(s ? s.roughness : null);
    this.metal.set(s ? s.metalness : null);
    this.sheen.set(s ? s.sheen : null);
    this.opacity.set(s ? s.opacity : null);
    for (const x of [this.rough, this.metal, this.sheen, this.opacity]) x.disable(off);
    this.scale.set(s ? s.map_scale : null);
    this.scale.disable(off);
    this.rot.set(s ? s.map_rotation : null);
    this.rot.disable(off);
    this.offset.set(s ? s.map_offset : null);
    this.offset.disable(off);
    for (const [key, row] of this.maps) {
      const path = s?.maps[key] ?? '';
      if (document.activeElement !== row.input && !row.input.classList.contains('is-bad')) setValue(row.input, path);
      if (row.input.disabled !== off) row.input.disabled = off;
      row.pick.disabled = off;
      row.clear.disabled = off || path === '';
      if (row.shown !== path) {
        row.shown = path;
        row.img.hidden = true;
        void previewUrl(path).then((url) => {
          if (row.shown !== path) return;
          if (url) { row.img.src = url; row.img.hidden = false; } else row.img.removeAttribute('src');
        });
      }
    }
  }
}
