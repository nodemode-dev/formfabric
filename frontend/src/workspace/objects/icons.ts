import { ICONS } from '../../ui/icons';

const S = 'viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"';

Object.assign(ICONS, {
  objects: `<svg ${S}><rect x="4" y="3.5" width="6" height="5" rx="1.2"/><path d="M7 8.5v8.2h4.2M7 12.5h4.2"/><rect x="12" y="10" width="8" height="5" rx="1.2"/><rect x="12" y="16" width="8" height="5" rx="1.2"/></svg>`,
  obCopy: `<svg ${S}><rect x="8.5" y="8.5" width="11" height="11" rx="1.6"/><path d="M15.5 8.5V6a1.6 1.6 0 0 0-1.6-1.6H6A1.6 1.6 0 0 0 4.4 6v7.9A1.6 1.6 0 0 0 6 15.5h2.5"/></svg>`,
  obMerge: `<svg ${S}><path d="M5 5l7 7v7M19 5l-7 7"/></svg>`,
  obUp: `<svg ${S}><path d="M12 19V6M6.5 11.5L12 6l5.5 5.5"/></svg>`,
  obDown: `<svg ${S}><path d="M12 5v13M6.5 12.5L12 18l5.5-5.5"/></svg>`,
});
