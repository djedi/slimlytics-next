import { createHash } from 'node:crypto';
import type { RequestHandler } from './$types';
import trackerBundle from '$lib/server/generated/tracker.iife.txt?raw';
import { trackerBootstrapSource } from '$lib/anti-adblock';

export const GET: RequestHandler = async ({ params, request }) => {
  try {
    const source = trackerBootstrapSource(trackerBundle, params.writeKey, `/${params.beacon}`);
    const etag = `"${createHash('sha256').update(source).digest('hex')}"`;
    if (request.headers.get('if-none-match') === etag) {
      return new Response(null, { status: 304, headers: { etag } });
    }
    return new Response(source, {
      headers: {
        'content-type': 'text/javascript; charset=utf-8',
        'cache-control': 'public, max-age=0, must-revalidate',
        'x-content-type-options': 'nosniff',
        etag
      }
    });
  } catch {
    return new Response('Invalid first-party tracker path.', {
      status: 400,
      headers: { 'content-type': 'text/plain; charset=utf-8' }
    });
  }
};
