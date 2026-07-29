import { describe, expect, it } from 'vitest';
import { trackingCode } from '../src/lib/tracking-code';

describe('tracking installation code', () => {
  it('generates the standard tracker snippet', () => {
    expect(trackingCode('https://stats.example.com/', 'site-key', false)).toBe(
      '<script async src="https://stats.example.com/tracker.js" data-write-key="site-key" data-endpoint="https://stats.example.com/api/collect"></script>'
    );
  });

  it('generates anti-adblock code with neutral request paths', () => {
    const snippet = trackingCode('https://stats.example.com', 'site-key', true);
    expect(snippet).toContain('src="https://stats.example.com/s.js"');
    expect(snippet).toContain('data-endpoint="https://stats.example.com/api/e"');
    expect(snippet).not.toContain('tracker');
    expect(snippet).not.toContain('collect');
  });

  it('escapes attribute values instead of emitting executable markup', () => {
    expect(trackingCode('https://stats.example.com', '\" onload=\"alert(1)', true))
      .toContain('data-write-key="&quot; onload=&quot;alert(1)"');
  });
});
