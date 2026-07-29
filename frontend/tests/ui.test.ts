import { describe, expect, it } from 'vitest';
import { applyTheme, sparklinePoints } from '../src/lib/ui';

describe('UI helpers', () => {
  it('creates bounded SVG points including flat datasets', () => {
    expect(sparklinePoints([5, 5, 5], 100, 20)).toBe('0,10 50,10 100,10');
    expect(sparklinePoints([0, 10], 100, 20)).toBe('0,20 100,0');
  });

  it('applies accessible light, dark, and system theme modes', () => {
    applyTheme('dark');
    expect(document.documentElement.dataset.theme).toBe('dark');
    applyTheme('light');
    expect(document.documentElement.dataset.theme).toBe('light');
    applyTheme('system');
    expect(document.documentElement.dataset.theme).toBeUndefined();
  });
});
