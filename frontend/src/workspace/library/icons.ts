import { ICONS } from '../../ui/icons';

const S = 'viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"';

Object.assign(ICONS, {
  libBrowse: `<svg ${S}><rect x="3.5" y="4" width="7" height="7" rx="1.2"/><rect x="13.5" y="4" width="7" height="7" rx="1.2"/><rect x="3.5" y="13" width="7" height="7" rx="1.2"/><path d="M13.5 16.5h7M13.5 19.5h4.5"/></svg>`,
  libSnapshot: `<svg ${S}><path d="M4 8.2A1.7 1.7 0 0 1 5.7 6.5h2l1.3-2h6l1.3 2h2A1.7 1.7 0 0 1 20 8.2v9.1a1.7 1.7 0 0 1-1.7 1.7H5.7A1.7 1.7 0 0 1 4 17.3z"/><circle cx="12" cy="12.7" r="3.4"/></svg>`,
  libComponent: `<svg ${S}><path d="M9 3.5v3.2M15 3.5v3.2"/><rect x="6" y="6.7" width="12" height="7.5" rx="1.6"/><path d="M12 14.2v3.1"/><path d="M8.5 20.5h7"/><path d="M12 17.3v3.2"/></svg>`,
  libKeys: `<svg ${S}><rect x="2.8" y="6" width="18.4" height="12" rx="2"/><path d="M6.5 10h.01M10 10h.01M13.5 10h.01M17 10h.01M7 14h10"/></svg>`,
  libLayout: `<svg ${S}><rect x="3.5" y="4.5" width="17" height="15" rx="2"/><path d="M9.5 4.5v15M9.5 11h11"/></svg>`,
  libPort: `<svg ${S}><path d="M3.5 12h5"/><circle cx="12" cy="12" r="3.2"/><path d="M15.2 12h5.3"/></svg>`,
  libSwap: `<svg ${S}><path d="M4 8h13l-3-3M20 16H7l3 3"/></svg>`,
  libJoin: `<svg ${S}><path d="M8 8.5a3.5 3.5 0 0 0 0 7h2M16 8.5a3.5 3.5 0 0 1 0 7h-2M9.5 12h5"/></svg>`,
  libCompare: `<svg ${S}><path d="M12 4v16"/><path d="M4.5 8h5M4.5 12h5M14.5 12h5M14.5 16h5"/></svg>`,
  libRestore: `<svg ${S}><path d="M4 12a8 8 0 1 0 2.5-5.8"/><path d="M4 4v4h4"/></svg>`,
});

export const KIND_ICON: Record<string, string> = {
  fabric: 'fabric',
  trim: 'trims',
  style: 'curve',
  component: 'libComponent',
  garment: 'panel',
  pose: 'body',
  motion: 'tlTimeline',
  place: 'libPort',
};
