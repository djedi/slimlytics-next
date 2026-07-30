import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtempSync, mkdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

const verifier = join(process.cwd(), 'scripts/verify-api-docs-build.mjs');

function fixture(css: string) {
  const root = mkdtempSync(join(tmpdir(), 'slimlytics-api-docs-build-'));
  const client = join(root, 'client');
  const generated = join(root, 'app.js');
  mkdirSync(join(client, '.vite'), { recursive: true });
  mkdirSync(join(client, '_app/immutable/assets'), { recursive: true });
  writeFileSync(generated, 'export const dictionary = { "/docs/api": [3] };');
  writeFileSync(
    join(client, '.vite/manifest.json'),
    JSON.stringify({ '.svelte-kit/generated/client-optimized/nodes/3.js': { css: ['_app/immutable/assets/api.css'] } })
  );
  writeFileSync(join(client, '_app/immutable/assets/api.css'), css);
  return { client, generated };
}

describe('API docs production build contract', () => {
  it('accepts emitted Scalar styles attached to the /docs/api route node', () => {
    const { client, generated } = fixture('.scalar-api-reference{}.references-layout{}');
    const result = spawnSync(process.execPath, [verifier, client, generated], { encoding: 'utf8' });
    expect(result.status, result.stderr).toBe(0);
    expect(result.stdout).toContain('Verified Scalar CSS');
  });

  it('reports the exact digest of the emitted route-associated Scalar asset', () => {
    const css = '.scalar-api-reference{}.references-layout{}';
    const { client, generated } = fixture(css);
    const result = spawnSync(process.execPath, [verifier, '--json', client, generated], { encoding: 'utf8' });
    expect(result.status, result.stderr).toBe(0);
    expect(JSON.parse(result.stdout)).toEqual([
      {
        asset: '_app/immutable/assets/api.css',
        sha256: createHash('sha256').update(css).digest('hex')
      }
    ]);
  });

  it('rejects an API docs build that omits Scalar styles', () => {
    const { client, generated } = fixture('body{margin:0}');
    const result = spawnSync(process.execPath, [verifier, client, generated], { encoding: 'utf8' });
    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain('do not contain the Scalar reference styles');
  });
});
