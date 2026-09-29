import type { Lang } from './i18n';

/** Text that is either language-independent or translated. */
export type Text = string | { it: string; en: string };

export interface SelectOption {
  value: string;
  label: Text;
  group?: string;
  hint?: string;
}

interface FieldBase {
  key: string;
  label: Text;
  depends_on?: string[];
}

export type Field =
  | (FieldBase & { type: 'select'; options: SelectOption[]; default?: string })
  | (FieldBase & { type: 'remote_select'; searchable: boolean });

export interface Step {
  id: string;
  title: Text;
  description?: Text;
  fields: Field[];
}

export interface University {
  id: string;
  name: Text;
  website?: string;
  steps: Step[];
  weeks: { min: number; max: number; default: number };
}

export interface Lesson {
  id: string;
  start: string;
  end: string;
  subject: string;
  teacher?: string;
  location?: string;
  notes?: string;
}

export interface LessonsResponse {
  /** IANA timezone of the university, for displaying times. */
  timezone: string;
  from: string;
  to: string;
  lessons: Lesson[];
}

/** Base path without trailing slash: "" at the root, "/timetable" under a prefix. */
export const BASE = import.meta.env.BASE_URL.replace(/\/$/, '');

export function localize(text: Text, lang: Lang): string {
  return typeof text === 'string' ? text : (text[lang] ?? text.it);
}

/**
 * Marks our own API calls: the backend doesn't track them (only calls from other clients count
 * as API usage). Must match `OWN_FRONTEND_HEADER` in `backend/src/state/tracking.rs`.
 */
const OWN_FRONTEND = { 'X-TT-Client': 'web' } as const;

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

async function get<T>(path: string, params?: Record<string, string>, signal?: AbortSignal): Promise<T> {
  const query = params ? `?${new URLSearchParams(params)}` : '';
  const res = await fetch(`${BASE}/api${path}${query}`, { signal, headers: OWN_FRONTEND });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new ApiError(res.status, body?.message ?? res.statusText);
  }
  return res.json();
}

export const api = {
  universities: (signal?: AbortSignal) => get<University[]>('/universities', undefined, signal),
  options: (uni: string, field: string, params: Record<string, string>, signal?: AbortSignal) =>
    get<SelectOption[]>(`/universities/${uni}/options/${field}`, params, signal),
  lessons: (uni: string, params: Record<string, string>, signal?: AbortSignal) =>
    get<LessonsResponse>(`/universities/${uni}/lessons`, params, signal),
};

export type RedisClear =
  | { status: 'disabled' | 'not_connected' }
  | { status: 'cleared'; keys: number }
  | { status: 'failed'; error: string };

export interface ClearReport {
  /** In-memory entries removed. */
  memory: number;
  redis: RedisClear;
}

/** `DELETE /api/admin/cache` with the `AUTH_TOKEN` as Bearer token. */
export async function clearCache(token: string): Promise<ClearReport> {
  const res = await fetch(`${BASE}/api/admin/cache`, {
    method: 'DELETE',
    headers: { ...OWN_FRONTEND, Authorization: `Bearer ${token}` },
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new ApiError(res.status, body?.message ?? res.statusText);
  }
  return res.json();
}

/** `POST /api/short`: the short link code for these settings (the same settings always get the same code). */
export async function createShortLink(uni: string, params: Record<string, string>, weeks: number): Promise<string> {
  const res = await fetch(`${BASE}/api/short`, {
    method: 'POST',
    headers: { ...OWN_FRONTEND, 'Content-Type': 'application/json' },
    body: JSON.stringify({ uni, params, weeks }),
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new ApiError(res.status, body?.message ?? res.statusText);
  }
  return ((await res.json()) as { code: string }).code;
}

/** Absolute URL of a short link. */
export function shortUrl(code: string): string {
  return `${location.origin}${BASE}/s/${code}`;
}

/**
 * The same short link for QR codes: uppercase (scheme, host, `/S/` and code are case-insensitive)
 * fits the denser QR alphanumeric mode. The base path keeps its case, routing needs it.
 */
export function shortUrlForQr(code: string): string {
  return `${location.origin.toUpperCase()}${BASE}/S/${code}`;
}

const TRACK_ID_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
let trackIdValue: string | undefined;

/**
 * Random id (8 × `[A-Za-z0-9]`) added to the feed URL as `k`, to count distinct subscriptions.
 * One per page load, kept in memory only: never in the wizard URL or in storage, so whoever opens
 * a shared link gets their own.
 */
function trackId(): string {
  if (!trackIdValue) {
    const bytes = crypto.getRandomValues(new Uint8Array(8));
    // 256 % 62 ≠ 0: the slight bias doesn't matter for a counter.
    trackIdValue = Array.from(bytes, (b) => TRACK_ID_CHARS[b % TRACK_ID_CHARS.length]).join('');
  }
  return trackIdValue;
}

/** Absolute URL of the ICS feed for the given parameters (plus the track id `k`). */
export function icsUrl(uni: string, params: Record<string, string>): string {
  const query = new URLSearchParams({ ...params, k: trackId() });
  return `${location.origin}${BASE}/api/universities/${uni}/lessons.ics?${query}`;
}
