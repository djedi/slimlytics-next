import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it, vi } from 'vitest';

const writeKey = 'd8f6f152-7a9e-4eb9-a8a1-468db4c0ea33';
const event = (writeKeyValue: string, beacon: string) => ({
  params: { writeKey: writeKeyValue, beacon },
  request: new Request(`https://slimlytics.com/p/${writeKeyValue}/${beacon}`)
}) as never;

describe('first-party tracker bootstrap route', () => {
  it('serves the complete tracker initialized for the exact beacon path', async () => {
    const { GET } = await import('../src/routes/p/[writeKey]/[beacon]/+server');
    const response = await GET(event(writeKey, '0d31360a3101'));
    expect(response.status).toBe(200);
    expect(response.headers.get('content-type')).toContain('text/javascript');
    expect(response.headers.get('x-content-type-options')).toBe('nosniff');
    const body = await response.text();
    expect(body).toContain('window.Slimlytics');
    expect(body).toContain('"endpoint":"/0d31360a3101"');
    expect(body).toContain('"appendWriteKey":false');
  });

  it('rejects unsafe route values', async () => {
    const { GET } = await import('../src/routes/p/[writeKey]/[beacon]/+server');
    const response = await GET(event('bad-key', '..'));
    expect(response.status).toBe(400);
  });

  it('changes its validator when a rotated write key changes the initializer', async () => {
    const { GET } = await import('../src/routes/p/[writeKey]/[beacon]/+server');
    const first = await GET(event(writeKey, '0d31360a3101'));
    const previousEtag = first.headers.get('etag')!;
    const rotatedKey = '5fd9aefc-1d8b-4cc8-b3d7-8f96931ba62e';
    const rotatedEvent = {
      params: { writeKey: rotatedKey, beacon: '0d31360a3101' },
      request: new Request(`https://slimlytics.com/p/${rotatedKey}/0d31360a3101`, {
        headers: { 'if-none-match': previousEtag }
      })
    } as never;
    const rotated = await GET(rotatedEvent);
    expect(rotated.status).toBe(200);
    expect(rotated.headers.get('etag')).not.toBe(previousEtag);
    expect(await rotated.text()).toContain(rotatedKey);
  });

  it('does not depend on the process working directory at runtime', async () => {
    const originalCwd = process.cwd();
    const directory = await mkdtemp(join(tmpdir(), 'slimlytics-route-'));
    try {
      process.chdir(directory);
      vi.resetModules();
      const { GET } = await import('../src/routes/p/[writeKey]/[beacon]/+server');
      const response = await GET(event(writeKey, '0d31360a3101'));
      expect(response.status).toBe(200);
      expect(await response.text()).toContain('"appendWriteKey":false');
    } finally {
      process.chdir(originalCwd);
      await rm(directory, { recursive: true, force: true });
    }
  });
});
