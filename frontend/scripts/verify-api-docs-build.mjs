#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

export function verifyApiDocsBuild(clientOutput, generatedApp) {
  const dictionary = readFileSync(generatedApp, 'utf8');
  const routeMatch = dictionary.match(/["']\/docs\/api["']\s*:\s*\[(\d+)\]/);
  if (!routeMatch) throw new Error('Could not resolve the /docs/api SvelteKit node');

  const nodeIndex = routeMatch[1];
  const manifestPath = resolve(clientOutput, '.vite/manifest.json');
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  const manifestEntry = Object.entries(manifest).find(([key]) => key.endsWith(`/nodes/${nodeIndex}.js`));
  if (!manifestEntry) throw new Error(`No client manifest entry for /docs/api node ${nodeIndex}`);

  const cssAssets = manifestEntry[1].css ?? [];
  if (cssAssets.length === 0) throw new Error('/docs/api has no emitted CSS assets');

  const scalarAssets = cssAssets.flatMap((asset) => {
    const bytes = readFileSync(resolve(clientOutput, asset));
    const css = bytes.toString('utf8');
    if (!css.includes('.scalar-api-reference') || !css.includes('.references-layout')) return [];
    return [{ asset, sha256: createHash('sha256').update(bytes).digest('hex') }];
  });
  if (scalarAssets.length === 0) {
    throw new Error('/docs/api CSS assets do not contain the Scalar reference styles');
  }

  return scalarAssets;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const jsonOutput = args.includes('--json');
  const positional = args.filter((arg) => arg !== '--json');
  const frontendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
  const clientOutput = positional[0]
    ? resolve(positional[0])
    : resolve(frontendRoot, '.svelte-kit/output/client');
  const generatedApp = positional[1]
    ? resolve(positional[1])
    : resolve(frontendRoot, '.svelte-kit/generated/client-optimized/app.js');
  const assets = verifyApiDocsBuild(clientOutput, generatedApp);
  if (jsonOutput) {
    console.log(JSON.stringify(assets));
  } else {
    console.log(`Verified Scalar CSS in /docs/api build assets: ${assets.map(({ asset }) => asset).join(', ')}`);
  }
}
