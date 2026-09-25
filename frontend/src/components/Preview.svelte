<script lang="ts">
  import { api, type Lesson, type LessonsResponse } from '../lib/api';
  import { useT, type Lang } from '../lib/i18n';

  let {
    uniId,
    params,
    lang,
  }: {
    uniId: string;
    params: Record<string, string>;
    lang: Lang;
  } = $props();

  /** Grid scale. */
  const HOUR_PX = 48;
  const COLORS = [
    'bg-green-600',
    'bg-sky-600',
    'bg-amber-500',
    'bg-violet-600',
    'bg-rose-600',
    'bg-teal-600',
    'bg-indigo-600',
    'bg-orange-500',
  ];

  const t = $derived(useT(lang));
  const locale = $derived(lang === 'it' ? 'it-IT' : 'en-GB');

  let data = $state<LessonsResponse | null>(null);
  let error = $state<string | null>(null);
  let selected = $state<string | null>(null);

  $effect(() => {
    const controller = new AbortController();
    data = null;
    error = null;
    selected = null;
    api
      .lessons(uniId, { ...params, weeks: '1' }, controller.signal)
      .then((res) => (data = res))
      .catch((e) => {
        if (!controller.signal.aborted) error = e.message || t('error.load');
      });
    return () => controller.abort();
  });

  interface Block {
    lesson: Lesson;
    top: number;
    height: number;
    lane: number;
    lanes: number;
    color: string;
    time: string;
  }

  interface Day {
    key: string;
    label: string;
    today: boolean;
    blocks: Block[];
  }

  const week = $derived.by(() => {
    if (!data) return null;
    const timeZone = data.timezone;
    const dayKey = new Intl.DateTimeFormat('en-CA', { timeZone, year: 'numeric', month: '2-digit', day: '2-digit' });
    const clock = new Intl.DateTimeFormat('en-GB', { timeZone, hour: '2-digit', minute: '2-digit', hourCycle: 'h23' });
    const dayLabel = new Intl.DateTimeFormat(locale, { timeZone, weekday: 'short', day: 'numeric' });
    const rangeLabel = new Intl.DateTimeFormat(locale, { timeZone, day: 'numeric', month: 'short' });
    const minutes = (d: Date) => {
      const [h, m] = clock.format(d).split(':').map(Number);
      return h * 60 + m;
    };

    // Monday..Sunday of the previewed week, in the university timezone.
    const monday = new Date(data.from);
    const dates = Array.from({ length: 7 }, (_, i) => new Date(monday.getTime() + i * 86_400_000 + 12 * 3_600_000));
    const todayKey = dayKey.format(new Date());

    const lessons = data.lessons
      .map((lesson) => ({ lesson, start: new Date(lesson.start), end: new Date(lesson.end) }))
      .map((l) => ({ ...l, day: dayKey.format(l.start), from: minutes(l.start), to: Math.max(minutes(l.end), minutes(l.start) + 15) }));

    const firstHour = Math.min(8, ...lessons.map((l) => Math.floor(l.from / 60)));
    const lastHour = Math.max(18, ...lessons.map((l) => Math.ceil(l.to / 60)));

    const subjects = [...new Set(lessons.map((l) => l.lesson.subject))];
    const color = (subject: string) => COLORS[subjects.indexOf(subject) % COLORS.length];

    const days: Day[] = dates.map((date) => {
      const key = dayKey.format(date);
      const own = lessons.filter((l) => l.day === key).sort((a, b) => a.from - b.from || b.to - a.to);

      // Side-by-side lanes for overlapping lessons: each cluster of overlaps shares its lane count.
      const blocks: Block[] = [];
      let cluster: Block[] = [];
      let laneEnds: number[] = [];
      let clusterEnd = -1;
      const flush = () => {
        for (const b of cluster) b.lanes = laneEnds.length;
        cluster = [];
        laneEnds = [];
      };
      for (const l of own) {
        if (l.from >= clusterEnd) flush();
        let lane = laneEnds.findIndex((end) => end <= l.from);
        if (lane === -1) lane = laneEnds.push(l.to) - 1;
        else laneEnds[lane] = l.to;
        clusterEnd = Math.max(clusterEnd, l.to);
        const block: Block = {
          lesson: l.lesson,
          top: ((l.from - firstHour * 60) / 60) * HOUR_PX,
          height: Math.max(((l.to - l.from) / 60) * HOUR_PX - 2, 18),
          lane,
          lanes: 1,
          color: color(l.lesson.subject),
          time: `${clock.format(l.start)}–${clock.format(l.end)}`,
        };
        cluster.push(block);
        blocks.push(block);
      }
      flush();
      return { key, label: dayLabel.format(date), today: key === todayKey, blocks };
    });

    // Always show Monday–Friday; weekend only when it has lessons.
    const visible = days.filter((d, i) => i < 5 || d.blocks.length > 0);
    return {
      days: visible,
      hours: Array.from({ length: lastHour - firstHour }, (_, i) => firstHour + i),
      height: (lastHour - firstHour) * HOUR_PX,
      range: `${rangeLabel.format(dates[0])} – ${rangeLabel.format(dates[6])}`,
      empty: lessons.length === 0,
    };
  });

  const detail = $derived(week?.days.flatMap((d) => d.blocks).find((b) => b.lesson.id === selected) ?? null);
</script>

<div>
  {#if week}<p class="-mt-1 mb-3 text-xs text-neutral-400">{week.range}</p>{/if}

  {#if error}
    <p class="rounded-lg bg-red-50 px-4 py-3 text-sm text-red-700">{error}</p>
  {:else if !week}
    <div class="h-72 animate-pulse rounded-lg bg-neutral-100" aria-label={t('preview.loading')}></div>
  {:else if week.empty}
    <p class="rounded-lg bg-neutral-50 px-4 py-10 text-center text-sm text-neutral-500">{t('preview.empty')}</p>
  {:else}
    <div class="overflow-x-auto">
      <div class="grid min-w-lg gap-1 text-[11px] sm:gap-2" style:grid-template-columns={`2.25rem repeat(${week.days.length}, minmax(0, 1fr))`}>
        <span></span>
        {#each week.days as day (day.key)}
          <p class="truncate text-center capitalize {day.today ? 'font-semibold text-accent' : 'text-neutral-400'}">{day.label}</p>
        {/each}

        <div class="relative" style:height={`${week.height}px`}>
          {#each week.hours as hour, i (hour)}
            <span class="absolute right-1 -translate-y-1/2 text-[10px] text-neutral-400" style:top={`${i * HOUR_PX}px`}>
              {String(hour).padStart(2, '0')}
            </span>
          {/each}
        </div>

        {#each week.days as day (day.key)}
          <div class="relative rounded-lg {day.today ? 'bg-accent/5' : 'bg-neutral-50'}" style:height={`${week.height}px`}>
            {#each week.hours.slice(1) as hour, i (hour)}
              <div class="absolute inset-x-0 border-t border-neutral-200/70" style:top={`${(i + 1) * HOUR_PX}px`}></div>
            {/each}
            {#each day.blocks as block (block.lesson.id)}
              <button
                type="button"
                class="absolute flex cursor-pointer flex-col justify-start overflow-hidden rounded-md px-1 py-0.5 text-left leading-tight text-white shadow-sm transition-opacity {block.color} {selected &&
                selected !== block.lesson.id
                  ? 'opacity-50'
                  : ''}"
                style:top={`${block.top}px`}
                style:height={`${block.height}px`}
                style:left={`calc(${(block.lane / block.lanes) * 100}% + 2px)`}
                style:width={`calc(${100 / block.lanes}% - 4px)`}
                title={`${block.time} ${block.lesson.subject}`}
                onclick={() => (selected = selected === block.lesson.id ? null : block.lesson.id)}
              >
                <span class="block opacity-90">{block.time.split('–')[0]}</span>
                <b class="line-clamp-3 font-semibold">{block.lesson.subject}</b>
              </button>
            {/each}
          </div>
        {/each}
      </div>
    </div>

    {#if detail}
      <div class="mt-4 rounded-lg border-l-4 border-accent bg-neutral-50 px-4 py-3 text-sm">
        <p class="font-mono text-xs text-neutral-500">{detail.time}</p>
        <p class="mt-0.5 font-medium">{detail.lesson.subject}</p>
        {#if detail.lesson.location || detail.lesson.teacher}
          <p class="text-xs text-neutral-500">{[detail.lesson.location, detail.lesson.teacher].filter(Boolean).join(' · ')}</p>
        {/if}
        {#if detail.lesson.notes}
          <p class="mt-1 text-xs text-amber-700 italic">{detail.lesson.notes}</p>
        {/if}
      </div>
    {/if}
  {/if}
</div>
