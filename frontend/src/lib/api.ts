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
  const res = await fetch(`${BASE}/api${path}${query}`, { signal });
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
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new ApiError(res.status, body?.message ?? res.statusText);
  }
  return res.json();
}

/** Absolute URL of the ICS feed for the given parameters. */
export function icsUrl(uni: string, params: Record<string, string>): string {
  return `${location.origin}${BASE}/api/universities/${uni}/lessons.ics?${new URLSearchParams(params)}`;
}
