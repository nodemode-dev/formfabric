export const isMac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

export function fromHtml<T extends HTMLElement = HTMLElement>(markup: string): T {
  const tpl = document.createElement('template');
  tpl.innerHTML = markup.trim();
  return tpl.content.firstElementChild as T;
}

export function q<T extends Element = HTMLElement>(root: ParentNode, selector: string): T {
  const node = root.querySelector(selector);
  if (!node) throw new Error(`missing element: ${selector}`);
  return node as T;
}

export function setText(node: Node, value: string): void {
  if (node.textContent !== value) node.textContent = value;
}

export function setClass(node: Element | null, cls: string, on: boolean): void {
  if (node && node.classList.contains(cls) !== on) node.classList.toggle(cls, on);
}

export function setHidden(node: HTMLElement, hidden: boolean): void {
  if (node.hidden !== hidden) node.hidden = hidden;
}

export function setDisabled(node: HTMLButtonElement | HTMLInputElement | HTMLSelectElement, disabled: boolean): void {
  if (node.disabled !== disabled) node.disabled = disabled;
}

export function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c] ?? c);
}
