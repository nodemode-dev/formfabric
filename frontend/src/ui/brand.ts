let copies = 0;

export function brandMark(size = 24, label = 'FormFabric'): string {
  const id = `ffb-mark-${++copies}`;
  const strand = `stroke="url(#${id})" stroke-width="3" stroke-linecap="butt" fill="none"`;
  return `<svg class="cv-mark" viewBox="0 0 24 24" width="${size}" height="${size}" role="img" aria-label="${label}">` +
    `<defs><linearGradient id="${id}" gradientUnits="userSpaceOnUse" x1="3" y1="21" x2="21" y2="3">` +
    '<stop offset="0" stop-color="#f97316"/><stop offset="0.35" stop-color="#ffb347"/><stop offset="0.7" stop-color="#e8577a"/><stop offset="1" stop-color="#7c5cff"/></linearGradient></defs>' +
    `<path ${strand} d="M3 8H13M19 8H21M3 16H5M11 16H21"/>` +
    `<path ${strand} d="M8 3V5M8 11V21M16 3V13M16 19V21"/>` +
    '</svg>';
}
