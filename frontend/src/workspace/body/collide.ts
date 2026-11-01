import type { Ctx } from './ctx';
import { slideRow, type SlideRow } from './widgets';

export class CollisionSection {
  private readonly offset: SlideRow;
  private readonly friction: SlideRow;

  constructor(host: HTMLElement, private readonly ctx: Ctx) {
    this.offset = slideRow('Skin offset', 0, 100, 0.5, (v) => this.send({ skin_offset: v }));
    this.friction = slideRow('Friction', 0, 10, 0.05, (v) => this.send({ friction: v }));
    host.append(this.offset.el, this.friction.el);
  }

  private send(args: Record<string, number>): void {
    const body = this.ctx.selected();
    if (body) void this.ctx.store.run('body.set', { id: body.id, ...args });
  }

  update(): void {
    const body = this.ctx.selected();
    this.offset.set(body ? body.skin_offset : null);
    this.friction.set(body ? body.friction : null);
    this.offset.disable(!body);
    this.friction.disable(!body);
  }
}
