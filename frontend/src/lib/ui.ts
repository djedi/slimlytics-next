export type Theme = 'light' | 'dark' | 'system';

export function sparklinePoints(values: number[], width: number, height: number): string {
  if (!values.length) return '';
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min;
  return values.map((value, index) => {
    const x = values.length === 1 ? width / 2 : (index / (values.length - 1)) * width;
    const y = range === 0 ? height / 2 : height - ((value - min) / range) * height;
    return `${Number(x.toFixed(2))},${Number(y.toFixed(2))}`;
  }).join(' ');
}

export function applyTheme(theme: Theme): void {
  if (theme === 'system') delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme === 'system' ? 'light dark' : theme;
}

export function duration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ${Math.round(seconds % 60)}s`;
}
