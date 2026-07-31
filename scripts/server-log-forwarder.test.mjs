import test from 'node:test';
import assert from 'node:assert/strict';
import { parseCaddyLine } from './server-log-forwarder.mjs';

const log = (userAgent, method = 'GET') => JSON.stringify({
  ts: 1785528000,
  status: 200,
  request: {
    method,
    uri: '/docs?utm_source=ai',
    client_ip: '203.0.113.20',
    headers: { 'User-Agent': [userAgent], Referer: ['https://search.example/'] }
  }
});

test('converts crawler access logs without retaining arbitrary fields', () => {
  const event = parseCaddyLine(log('GPTBot/1.0'), 'https://example.com');
  assert.equal(event.url, 'https://example.com/docs?utm_source=ai');
  assert.equal(event.clientIp, '203.0.113.20');
  assert.match(event.idempotencyKey, /^caddy:[a-f0-9]{64}$/);
  assert.equal(event.eventName, 'pageview');
});

test('defaults to crawler-only and permits explicit all-traffic mode', () => {
  assert.equal(parseCaddyLine(log('Mozilla/5.0'), 'https://example.com'), null);
  assert.ok(parseCaddyLine(log('Mozilla/5.0'), 'https://example.com', 'all'));
  assert.equal(parseCaddyLine(log('GPTBot/1.0', 'POST'), 'https://example.com'), null);
});
