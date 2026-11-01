import { ICONS } from '../../ui/icons';

const S = 'viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"';

Object.assign(ICONS, {
  trims: `<svg ${S}><path d="M4 12.2V5a1 1 0 0 1 1-1h7.2l8 8-8.2 8.2z"/><circle cx="8.6" cy="8.6" r="1.3"/></svg>`,
  tButton: `<svg ${S}><circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="4.6" stroke-opacity=".5"/><circle cx="10.2" cy="10.2" r=".7" fill="currentColor"/><circle cx="13.8" cy="10.2" r=".7" fill="currentColor"/><circle cx="10.2" cy="13.8" r=".7" fill="currentColor"/><circle cx="13.8" cy="13.8" r=".7" fill="currentColor"/></svg>`,
  tButtonhole: `<svg ${S}><rect x="3.5" y="9" width="17" height="6" rx="3"/><path d="M6.5 12h11" stroke-dasharray="1.4 1.8"/></svg>`,
  tZipper: `<svg ${S}><path d="M12 3.5v13"/><path d="M8.3 5.5H12M12 8H15.7M8.3 10.5H12M12 13H15.7"/><rect x="9.8" y="16.5" width="4.4" height="4" rx="1.2"/></svg>`,
  tTopstitch: `<svg ${S}><path d="M3.5 7h17"/><path d="M3.5 15.5h3.4M9 15.5h3.4M14.5 15.5h3.4M19.5 15.5h1"/><path d="M3.5 11h17" stroke-opacity=".35"/></svg>`,
  tFinish: `<svg ${S}><path d="M20.5 6.5H6A2.5 2.5 0 0 0 3.5 9v6A2.5 2.5 0 0 0 6 17.5h14.5"/><path d="M9 10.5h11.5M9 13.5h11.5" stroke-opacity=".6"/></svg>`,
  tPrint: `<svg ${S}><rect x="4" y="5" width="16" height="14" rx="1.6"/><circle cx="9" cy="10" r="1.5"/><path d="M4.5 17l4.5-4.5 3.5 3.5 2.5-2.5 4.5 3.5"/></svg>`,
  tFasten: `<svg ${S}><circle cx="7" cy="12" r="3.4"/><rect x="14" y="9.8" width="7" height="4.4" rx="2.2"/><path d="M10.6 12H14" stroke-dasharray="1.6 1.4"/></svg>`,
  tTrimSelect: `<svg ${S}><path d="M5.5 3.5l9.5 6.6-4.2.9 2.3 4.4-2 1-2.3-4.4L5.5 14.6z"/><circle cx="17.6" cy="17.6" r="2.6"/></svg>`,
  tPuckering: `<svg ${S}><path d="M3.5 12c1.6-5 3.2-5 4.8 0s3.2 5 4.8 0 3.2-5 4.8 0 2.4 3.4 2.6 3"/></svg>`,
  tTrimMesh: `<svg ${S}><path d="M12 3.5l7 4v9l-7 4-7-4v-9z"/><path d="M5 7.5l7 4 7-4M12 11.5v9"/></svg>`,
});

export const KIND_ICON: Record<string, string> = {
  button: 'tButton',
  buttonhole: 'tButtonhole',
  zipper: 'tZipper',
  topstitch: 'tTopstitch',
  puckering: 'tPuckering',
  trim: 'tTrimMesh',
  hardware: 'tButton',
  stitch: 'tTopstitch',
  finish: 'tFinish',
  print: 'tPrint',
};
