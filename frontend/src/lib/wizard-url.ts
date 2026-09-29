import { api, localize, type Field, type SelectOption, type University } from './api';
import type { Lang } from './i18n';

/**
 * Wizard progress. Everything but `labels` lives in the page URL, so a link reproduces the wizard:
 * `?uni=<id>&<field key>=<value>…&weeks=N&name=…&step=<step id>`.
 */
export interface Progress {
  uniId: string | null;
  values: Record<string, string>;
  /** Localized labels of the selected values, resolved from the schema or the remote options. */
  labels: Record<string, string>;
  weeks: number;
  name: string;
  /** The name was typed by the user: only then is it kept in the URL. */
  nameEdited: boolean;
  index: number;
}

export const EMPTY: Progress = { uniId: null, values: {}, labels: {}, weeks: 0, name: '', nameEdited: false, index: 0 };

/** `step=` values, one per wizard screen: the frontend's own steps around the schema steps. */
export function stepIds(uni: University | null): string[] {
  return ['university', ...(uni?.steps ?? []).map((s) => s.id), 'options', 'result'];
}

export function fields(uni: University): Field[] {
  return uni.steps.flatMap((s) => s.fields);
}

/** Values of the static selects that declare a default, with their labels. */
export function defaults(uni: University, lang: Lang): Pick<Progress, 'values' | 'labels'> {
  const values: Record<string, string> = {};
  const labels: Record<string, string> = {};
  for (const f of fields(uni)) {
    const def = f.type === 'select' && f.default ? f.options.find((o) => o.value === f.default) : undefined;
    if (def) {
      values[f.key] = def.value;
      labels[f.key] = localize(def.label, lang);
    }
  }
  return { values, labels };
}

/** Index of the furthest screen whose previous screens are all complete. */
export function furthestReachable(uni: University, p: Progress): number {
  let index = 1;
  for (const step of uni.steps) {
    if (!step.fields.every((f) => p.values[f.key])) return index;
    index++;
  }
  const optionsOk = p.weeks >= uni.weeks.min && p.weeks <= uni.weeks.max && p.name.trim() !== '';
  return optionsOk ? index + 1 : index;
}

/** The URL query for `p` (without `?`). */
export function toQuery(uni: University | null, p: Progress): string {
  const q = new URLSearchParams();
  if (uni) {
    q.set('uni', uni.id);
    for (const f of fields(uni)) if (p.values[f.key]) q.set(f.key, p.values[f.key]);
    q.set('weeks', String(p.weeks));
    if (p.nameEdited && p.name.trim()) q.set('name', p.name);
  }
  const step = stepIds(uni)[p.index];
  if (step && step !== 'university') q.set('step', step);
  return q.toString();
}

/**
 * Rebuilds the progress from a URL query. Values are validated against the schema and the remote
 * options (loading their labels); invalid ones are dropped with the fields that depend on them
 * (static defaults fill in when missing). The requested step is capped at the furthest one whose
 * previous steps are complete. `known` supplies labels already resolved, to skip reloading them.
 * `defaultName` builds the calendar name when the query has none.
 */
export async function fromQuery(
  query: string,
  universities: University[],
  lang: Lang,
  known: Progress,
  defaultName: (uni: University, labels: Record<string, string>) => string,
): Promise<Progress> {
  const q = new URLSearchParams(query);
  const uni = universities.find((u) => u.id === q.get('uni'));
  if (!uni) return structuredClone(EMPTY);

  const { values, labels } = defaults(uni, lang);
  for (const f of fields(uni)) {
    const value = q.get(f.key);
    if (!value || (f.depends_on ?? []).some((d) => !values[d])) continue;
    const label =
      known.uniId === uni.id && known.values[f.key] === value && known.labels[f.key]
        ? known.labels[f.key]
        : await resolveLabel(uni.id, f, value, values, lang);
    if (label === null) continue;
    values[f.key] = value;
    labels[f.key] = label;
  }

  const weeks = Number(q.get('weeks'));
  const name = q.get('name')?.trim() ?? '';
  const p: Progress = {
    uniId: uni.id,
    values,
    labels,
    weeks: Number.isInteger(weeks) && weeks >= uni.weeks.min && weeks <= uni.weeks.max ? weeks : uni.weeks.default,
    name: name || defaultName(uni, labels),
    nameEdited: name !== '',
    index: 0,
  };
  const requested = stepIds(uni).indexOf(q.get('step') ?? '');
  const reachable = furthestReachable(uni, p);
  p.index = requested < 0 ? reachable : Math.min(requested, reachable);
  return p;
}

/** Label of `value` among the field's options, or `null` if it isn't one of them. */
async function resolveLabel(
  uniId: string,
  field: Field,
  value: string,
  values: Record<string, string>,
  lang: Lang,
): Promise<string | null> {
  let options: SelectOption[];
  if (field.type === 'select') {
    options = field.options;
  } else {
    const deps = Object.fromEntries((field.depends_on ?? []).map((k) => [k, values[k]]));
    try {
      options = await api.options(uniId, field.key, deps);
    } catch {
      return null;
    }
  }
  const option = options.find((o) => o.value === value);
  return option ? localize(option.label, lang) : null;
}
