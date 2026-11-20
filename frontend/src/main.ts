import { startApp } from './app';
import { claimWindow } from './engine/service';
import { startLanguage, startPageTranslator } from './ui/i18n';

void claimWindow().finally(() => startLanguage().finally(() => { startApp(); startPageTranslator(); }));
