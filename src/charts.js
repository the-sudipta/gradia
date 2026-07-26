import { clamp, escapeHtml, formatNumber } from "./core.js";

export function barChart(items, { height = 220, color = "#9b7bff", valueSuffix = "" } = {}) {
  const width = 640;
  const padding = { top: 18, right: 18, bottom: 42, left: 42 };
  const plotWidth = width - padding.left - padding.right;
  const plotHeight = height - padding.top - padding.bottom;
  const maximum = Math.max(1, ...items.map((item) => Number(item.value) || 0));
  const step = items.length ? plotWidth / items.length : plotWidth;
  const barWidth = Math.max(10, step * 0.58);
  const bars = items
    .map((item, index) => {
      const value = Number(item.value) || 0;
      const barHeight = (value / maximum) * plotHeight;
      const x = padding.left + index * step + (step - barWidth) / 2;
      const y = padding.top + plotHeight - barHeight;
      return `
        <g class="chart-bar">
          <rect x="${x}" y="${y}" width="${barWidth}" height="${barHeight}" rx="5" fill="${item.color || color}"></rect>
          <text x="${x + barWidth / 2}" y="${Math.max(12, y - 7)}" text-anchor="middle" class="chart-value">${escapeHtml(formatNumber(value))}${valueSuffix}</text>
          <text x="${x + barWidth / 2}" y="${height - 16}" text-anchor="middle" class="chart-label">${escapeHtml(item.label)}</text>
        </g>`;
    })
    .join("");
  return `
    <svg class="chart" viewBox="0 0 ${width} ${height}" role="img" aria-label="Bar chart">
      <line x1="${padding.left}" y1="${padding.top + plotHeight}" x2="${width - padding.right}" y2="${padding.top + plotHeight}" class="chart-axis"></line>
      ${bars || `<text x="${width / 2}" y="${height / 2}" text-anchor="middle" class="chart-empty">No data yet</text>`}
    </svg>`;
}

export function distributionChart(results, { height = 230 } = {}) {
  const values = results.map((result) => result.final_percentage).filter(Number.isFinite);
  const bins = Array.from({ length: 10 }, (_, index) => ({
    label: `${index * 10}–${index * 10 + 9}`,
    value: 0
  }));
  for (const value of values) {
    bins[clamp(Math.floor(value / 10), 0, 9)].value += 1;
  }
  return barChart(bins, { height, color: "#7c5cff" });
}

export function donutChart(percent, color = "#8b5cf6") {
  const value = clamp(Number(percent) || 0, 0, 100);
  const radius = 42;
  const circumference = Math.PI * 2 * radius;
  return `
    <svg class="donut" viewBox="0 0 110 110" role="img" aria-label="${value.toFixed(0)} percent">
      <circle cx="55" cy="55" r="${radius}" class="donut-track"></circle>
      <circle cx="55" cy="55" r="${radius}" class="donut-value" style="stroke:${color};stroke-dasharray:${circumference};stroke-dashoffset:${circumference * (1 - value / 100)}"></circle>
      <text x="55" y="59" text-anchor="middle">${formatNumber(value, 0)}%</text>
    </svg>`;
}

export function sparkline(values, color = "#8b5cf6") {
  const points = values.map(Number).filter(Number.isFinite);
  if (points.length < 2) return "";
  const width = 160;
  const height = 48;
  const min = Math.min(...points);
  const max = Math.max(...points);
  const range = max - min || 1;
  const path = points
    .map((value, index) => {
      const x = (index / (points.length - 1)) * width;
      const y = height - ((value - min) / range) * (height - 8) - 4;
      return `${index ? "L" : "M"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
  return `<svg class="sparkline" viewBox="0 0 ${width} ${height}"><path d="${path}" style="stroke:${color}"></path></svg>`;
}
