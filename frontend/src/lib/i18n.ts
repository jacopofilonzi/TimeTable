export const LANGS = ['it', 'en'] as const;
export type Lang = (typeof LANGS)[number];
export const DEFAULT_LANG: Lang = 'it';

const it = {
  'meta.title': 'TimeTable — Orario universitario nel tuo calendario',
  'meta.description':
    "Crea il link per iscriverti all'orario delle lezioni della tua università da Google Calendar, Apple Calendar, Outlook e qualsiasi app compatibile con iCal.",
  'header.title': 'TimeTable',
  'header.subtitle': "Aggiungi l'orario della tua università a qualsiasi calendario compatibile con iCal",
  'footer.opensource': 'Progetto open-source, i contributi sono benvenuti.',
  'footer.rights': 'Tutti i diritti riservati.',
  'sidebar.tagline': 'Orario universitario nel tuo calendario',
  'summary.title': 'Riepilogo',
  'summary.university': 'Università',
  'summary.weeks': 'Settimane',
  'steps.counter': 'Passo {n} di {total}',

  'steps.university': 'Università',
  'steps.options': 'Opzioni',
  'steps.result': 'Link',

  'university.title': 'Scegli la tua università',
  'university.subtitle': "Il calendario viene generato dall'orario ufficiale pubblicato dall'ateneo.",
  'university.missing': 'La tua università non è presente?',
  'university.missingLink': 'Apri una issue su GitHub',

  'field.search': 'Cerca…',
  'field.noResults': 'Nessun risultato',
  'field.loading': 'Caricamento…',
  'field.retry': 'Riprova',

  'options.title': 'Opzioni del calendario',
  'options.weeks': 'Quante settimane vuoi vedere in avanti?',
  'options.weeksHint': "Il calendario mostra sempre le lezioni dalla settimana corrente in avanti, per il numero di settimane scelto.",
  'options.name': 'Nome del calendario',
  'options.nameHint': 'Come apparirà nella tua app calendario.',

  'result.done': 'Fatto',
  'result.title': 'Il tuo calendario è pronto',
  'result.subtitle': 'Iscriviti al link: il calendario si aggiornerà da solo quando cambia l\'orario.',
  'result.copy': 'Copia',
  'result.copied': 'Copiato!',
  'result.google': 'Google Calendar',
  'result.googleHint': 'Account Google',
  'result.apple': 'Apple Calendar',
  'result.appleHint': 'iPhone, iPad e Mac',
  'result.more': 'Altre app e formati',
  'result.webcal': 'webcal',
  'result.outlook': 'Outlook',
  'result.download': 'Scarica .ics',
  'result.qr': 'QR code',
  'result.preview': 'Anteprima',
  'result.qrTitle': 'Inquadra con il telefono',
  'result.qrHint': 'Apri il link sul telefono per iscriverti al calendario.',
  'result.qrError': 'Impossibile generare il QR code. Ricarica la pagina e riprova.',
  'result.close': 'Chiudi',
  'result.restart': 'Ricomincia',

  'preview.title': 'Anteprima di questa settimana',
  'preview.empty': 'Nessuna lezione in programma questa settimana.',
  'preview.loading': 'Caricamento anteprima…',

  'nav.back': 'Indietro',
  'nav.next': 'Avanti',
  'nav.create': 'Crea link',

  'admin.title': 'Amministrazione',
  'admin.cacheTitle': 'Svuota la cache',
  'admin.cacheDescription':
    'Rimuove orari e liste in cache (memoria e Redis): le prossime richieste verranno lette di nuovo dai siti delle università.',
  'admin.token': 'Token di accesso',
  'admin.tokenHint': 'Il valore di AUTH_TOKEN nel .env del server. Non viene salvato.',
  'admin.submit': 'Svuota cache',
  'admin.running': 'Svuotamento…',
  'admin.done': 'Cache svuotata',
  'admin.memory': 'Voci rimosse dalla memoria',
  'admin.redis': 'Redis',
  'admin.redisDisabled': 'non configurato',
  'admin.redisNotConnected': 'non connesso, nessuna chiave rimossa',
  'admin.redisCleared': '{n} chiavi rimosse',
  'admin.redisFailed': 'errore: {error}',
  'admin.unauthorized': 'Token non valido.',
  'admin.disabled': 'Funzione disabilitata: imposta AUTH_TOKEN nel .env del server.',
  'error.generic': 'Qualcosa è andato storto.',
  'error.load': 'Impossibile caricare i dati.',
};

type Dict = Record<keyof typeof it, string>;

const en: Dict = {
  'meta.title': 'TimeTable — University timetable in your calendar',
  'meta.description':
    "Build the link to subscribe to your university's lesson timetable from Google Calendar, Apple Calendar, Outlook and any iCal-compatible app.",
  'header.title': 'TimeTable',
  'header.subtitle': 'Add your university timetable to any iCal-compatible calendar',
  'footer.opensource': 'Open-source project, contributions are welcome.',
  'footer.rights': 'All rights reserved.',
  'sidebar.tagline': 'University timetable in your calendar',
  'summary.title': 'Summary',
  'summary.university': 'University',
  'summary.weeks': 'Weeks',
  'steps.counter': 'Step {n} of {total}',

  'steps.university': 'University',
  'steps.options': 'Options',
  'steps.result': 'Link',

  'university.title': 'Choose your university',
  'university.subtitle': 'The calendar is generated from the official timetable published by the university.',
  'university.missing': "Your university isn't listed?",
  'university.missingLink': 'Open an issue on GitHub',

  'field.search': 'Search…',
  'field.noResults': 'No results',
  'field.loading': 'Loading…',
  'field.retry': 'Retry',

  'options.title': 'Calendar options',
  'options.weeks': 'How many weeks ahead do you want to see?',
  'options.weeksHint': 'The calendar always shows lessons from the current week onwards, for the chosen number of weeks.',
  'options.name': 'Calendar name',
  'options.nameHint': 'How it will appear in your calendar app.',

  'result.done': 'Done',
  'result.title': 'Your calendar is ready',
  'result.subtitle': 'Subscribe to the link: the calendar updates itself when the timetable changes.',
  'result.copy': 'Copy',
  'result.copied': 'Copied!',
  'result.google': 'Google Calendar',
  'result.googleHint': 'Google account',
  'result.apple': 'Apple Calendar',
  'result.appleHint': 'iPhone, iPad and Mac',
  'result.more': 'Other apps and formats',
  'result.webcal': 'webcal',
  'result.outlook': 'Outlook',
  'result.download': 'Download .ics',
  'result.qr': 'QR code',
  'result.preview': 'Preview',
  'result.qrTitle': 'Scan with your phone',
  'result.qrHint': 'Open the link on your phone to subscribe to the calendar.',
  'result.qrError': 'Could not generate the QR code. Reload the page and try again.',
  'result.close': 'Close',
  'result.restart': 'Start over',

  'preview.title': "This week's preview",
  'preview.empty': 'No lessons scheduled this week.',
  'preview.loading': 'Loading preview…',

  'nav.back': 'Back',
  'nav.next': 'Next',
  'nav.create': 'Create link',

  'admin.title': 'Administration',
  'admin.cacheTitle': 'Clear the cache',
  'admin.cacheDescription':
    "Removes cached timetables and lists (memory and Redis): the next requests will be read again from the universities' websites.",
  'admin.token': 'Access token',
  'admin.tokenHint': "The value of AUTH_TOKEN in the server's .env. It is not stored.",
  'admin.submit': 'Clear cache',
  'admin.running': 'Clearing…',
  'admin.done': 'Cache cleared',
  'admin.memory': 'Entries removed from memory',
  'admin.redis': 'Redis',
  'admin.redisDisabled': 'not configured',
  'admin.redisNotConnected': 'not connected, no keys removed',
  'admin.redisCleared': '{n} keys removed',
  'admin.redisFailed': 'error: {error}',
  'admin.unauthorized': 'Invalid token.',
  'admin.disabled': "Feature disabled: set AUTH_TOKEN in the server's .env.",
  'error.generic': 'Something went wrong.',
  'error.load': 'Could not load data.',
};

const dicts: Record<Lang, Dict> = { it, en };

export type Key = keyof Dict;

export function useT(lang: Lang) {
  return (key: Key, vars?: Record<string, string | number>) => {
    let s = dicts[lang][key];
    for (const [k, v] of Object.entries(vars ?? {})) s = s.replace(`{${k}}`, String(v));
    return s;
  };
}
