export class Session {
  path: string | null = null;
  dirty = false;
  active = false;
  private readonly listeners = new Set<() => void>();
  private readonly KEY = 'formfabric.recent';

  name(): string {
    return this.path ? (this.path.split(/[\\/]/).pop() ?? this.path) : 'Untitled';
  }

  title(): string {
    return `${this.dirty ? '• ' : ''}${this.name()} - FormFabric`;
  }

  set(path: string | null, dirty: boolean): void {
    this.path = path;
    this.dirty = dirty;
    this.active = true;
    this.emit();
  }

  touch(): void {
    if (this.dirty) return;
    this.dirty = true;
    this.emit();
  }

  onChange(fn: () => void): void { this.listeners.add(fn); }

  private emit(): void { for (const fn of this.listeners) fn(); }

  recent(): string[] {
    try {
      const v: unknown = JSON.parse(localStorage.getItem(this.KEY) ?? '[]');
      return Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : [];
    } catch { return []; }
  }

  remember(path: string, limit: number): void {
    const next = [path, ...this.recent().filter((p) => p !== path)].slice(0, limit);
    try { localStorage.setItem(this.KEY, JSON.stringify(next)); } catch { return; }
    this.emit();
  }

  forget(path?: string): void {
    const next = path ? this.recent().filter((p) => p !== path) : [];
    try { localStorage.setItem(this.KEY, JSON.stringify(next)); } catch { return; }
    this.emit();
  }
}
