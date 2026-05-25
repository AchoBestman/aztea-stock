import { Sale } from '../services/api';

export type ChartPeriod = 'day' | 'week' | 'month' | 'interval';

export interface ChartSeries {
  labels: string[];
  values: number[];
}

const WEEK_LABELS = ['Lun', 'Mar', 'Mer', 'Jeu', 'Ven', 'Sam', 'Dim'];

export function buildRevenueChartSeries(
  sales: Sale[],
  period: ChartPeriod,
  dateRange: { start: string; end: string }
): ChartSeries {
  const completed = sales.filter((s) => s.status === 'completed');
  const now = new Date();

  if (period === 'week') {
    const startOfWeek = new Date(now);
    const day = startOfWeek.getDay();
    const diff = day === 0 ? -6 : 1 - day;
    startOfWeek.setDate(startOfWeek.getDate() + diff);
    startOfWeek.setHours(0, 0, 0, 0);

    const values = Array(7).fill(0);
    for (const sale of completed) {
      const d = new Date(sale.sold_at);
      if (d < startOfWeek) continue;
      const idx = Math.floor((d.getTime() - startOfWeek.getTime()) / 86400000);
      if (idx >= 0 && idx < 7) values[idx] += sale.total;
    }
    return { labels: WEEK_LABELS, values };
  }

  if (period === 'day') {
    const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const labels = ['06h', '09h', '12h', '15h', '18h', '21h'];
    const bucketHours = [6, 9, 12, 15, 18, 21];
    const values = Array(6).fill(0);
    for (const sale of completed) {
      const d = new Date(sale.sold_at);
      if (d < startOfToday) continue;
      const h = d.getHours();
      let idx = -1;
      for (let i = 0; i < bucketHours.length; i++) {
        const next = bucketHours[i + 1] ?? 24;
        if (h >= bucketHours[i] && h < next) {
          idx = i;
          break;
        }
      }
      if (idx < 0) idx = bucketHours.length - 1;
      values[idx] += sale.total;
    }
    return { labels, values };
  }

  if (period === 'month') {
    const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);
    const daysInMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0).getDate();
    const bucketCount = Math.min(7, daysInMonth);
    const step = Math.ceil(daysInMonth / bucketCount);
    const labels: string[] = [];
    const values = Array(bucketCount).fill(0);
    for (let i = 0; i < bucketCount; i++) {
      const dayNum = Math.min(1 + i * step, daysInMonth);
      labels.push(String(dayNum));
    }
    for (const sale of completed) {
      const d = new Date(sale.sold_at);
      if (d < startOfMonth) continue;
      const dayOfMonth = d.getDate() - 1;
      const idx = Math.min(Math.floor(dayOfMonth / step), bucketCount - 1);
      values[idx] += sale.total;
    }
    return { labels, values };
  }

  const start = new Date(`${dateRange.start}T00:00:00`);
  const end = new Date(`${dateRange.end}T23:59:59`);
  const msPerDay = 86400000;
  const dayCount = Math.min(
    14,
    Math.max(1, Math.floor((end.getTime() - start.getTime()) / msPerDay) + 1)
  );
  const labels: string[] = [];
  const values = Array(dayCount).fill(0);
  for (let i = 0; i < dayCount; i++) {
    const d = new Date(start);
    d.setDate(d.getDate() + i);
    labels.push(d.toLocaleDateString('fr-FR', { day: 'numeric', month: 'short' }));
  }
  for (const sale of completed) {
    const sold = new Date(sale.sold_at);
    if (sold < start || sold > end) continue;
    const idx = Math.floor((sold.getTime() - start.getTime()) / msPerDay);
    if (idx >= 0 && idx < dayCount) values[idx] += sale.total;
  }
  return { labels, values };
}

export function formatChartAxisValue(val: number): string {
  if (val >= 1_000_000) return `${Math.round(val / 1_000_000)}M F`;
  if (val >= 1000) return `${Math.round(val / 1000)}k F`;
  return `${Math.round(val)} F`;
}

export function buildChartPaths(
  values: number[],
  width = 640,
  height = 200
): { linePath: string; areaPath: string; points: { x: number; y: number }[] } {
  const plotH = height - 40;
  const plotW = width - 40;
  const left = 20;
  const bottom = 20 + plotH;
  const max = Math.max(...values, 0);
  const scaleMax = max > 0 ? max : 1;
  const n = values.length;

  if (n === 0) {
    return { linePath: '', areaPath: '', points: [] };
  }

  const points = values.map((v, i) => ({
    x: left + (n <= 1 ? plotW / 2 : (i / (n - 1)) * plotW),
    y: 20 + plotH * (1 - v / scaleMax),
  }));

  const linePath = points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x.toFixed(1)} ${p.y.toFixed(1)}`).join(' ');
  const areaPath = `${linePath} L ${points[points.length - 1].x.toFixed(1)} ${bottom} L ${points[0].x.toFixed(1)} ${bottom} Z`;

  return { linePath, areaPath, points };
}

export function getChartMaxValue(values: number[]): number {
  return Math.max(...values, 0);
}
