#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { createInterface } from 'node:readline';
import { pathToFileURL } from 'node:url';

const BOT_PATTERN = /(?:bot|spider|crawler|slurp|preview|GPTBot|ChatGPT-User|OAI-SearchBot|Claude(?:Bot|-User)|Perplexity(?:Bot|-User)|Google-Extended|Bytespider)/i;

function firstHeader(headers, name) {
  const value = headers?.[name] ?? headers?.[name.toLowerCase()];
  return Array.isArray(value) ? value[0] : value;
}

export function parseCaddyLine(line, siteOrigin, mode = 'bots') {
  const record = JSON.parse(line);
  const request = record.request ?? {};
  const method = String(request.method ?? '').toUpperCase();
  if (!['GET', 'HEAD'].includes(method)) return null;
  const userAgent = firstHeader(request.headers, 'User-Agent') ?? '';
  if (mode !== 'all' && !BOT_PATTERN.test(userAgent)) return null;
  const clientIp = request.client_ip ?? request.remote_ip;
  if (!clientIp) throw new Error('Caddy log record is missing request.client_ip');
  const uri = String(request.uri ?? '/');
  const url = new URL(uri, siteOrigin).toString();
  const occurredAt = new Date(Number(record.ts) * 1000).toISOString();
  const seed = `${record.ts}|${clientIp}|${userAgent}|${method}|${uri}|${record.status ?? ''}`;
  return {
    idempotencyKey: `caddy:${createHash('sha256').update(seed).digest('hex')}`,
    url,
    userAgent,
    clientIp,
    occurredAt,
    referrer: firstHeader(request.headers, 'Referer') || undefined,
    method,
    status: Number(record.status) || undefined,
    eventName: 'pageview'
  };
}

async function sendBatch(endpoint, serverKey, events) {
  const response = await fetch(endpoint, {
    method: 'POST',
    headers: { 'content-type': 'application/json', 'x-slimlytics-server-key': serverKey },
    body: JSON.stringify({ events })
  });
  if (!response.ok) throw new Error(`Slimlytics ingestion returned HTTP ${response.status}`);
  return response.json();
}

export async function run(environment = process.env, input = process.stdin) {
  const endpoint = environment.SLIMLYTICS_SERVER_INGEST_URL;
  const serverKey = environment.SLIMLYTICS_SERVER_WRITE_KEY;
  const siteOrigin = environment.SLIMLYTICS_SITE_ORIGIN;
  const mode = environment.SLIMLYTICS_LOG_MODE ?? 'bots';
  if (!endpoint || !serverKey || !siteOrigin) {
    throw new Error('SLIMLYTICS_SERVER_INGEST_URL, SLIMLYTICS_SERVER_WRITE_KEY, and SLIMLYTICS_SITE_ORIGIN are required');
  }
  if (!['bots', 'all'].includes(mode)) throw new Error('SLIMLYTICS_LOG_MODE must be bots or all');
  const events = [];
  for await (const line of createInterface({ input, crlfDelay: Infinity })) {
    if (!line.trim()) continue;
    const event = parseCaddyLine(line, siteOrigin, mode);
    if (event) events.push(event);
    if (events.length === 100) await sendBatch(endpoint, serverKey, events.splice(0));
  }
  if (events.length) await sendBatch(endpoint, serverKey, events);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  run().catch((error) => {
    process.stderr.write(`server-log-forwarder: ${error.message}\n`);
    process.exitCode = 1;
  });
}
