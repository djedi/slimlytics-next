function escapeAttribute(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('"', '&quot;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');
}

export function trackingCode(origin: string, writeKey: string, antiAdblock: boolean): string {
  const base = origin.replace(/\/$/, '');
  const scriptPath = antiAdblock ? '/s.js' : '/tracker.js';
  const collectionPath = antiAdblock ? '/api/e' : '/api/collect';
  return `<script async src="${escapeAttribute(base + scriptPath)}" data-write-key="${escapeAttribute(writeKey)}" data-endpoint="${escapeAttribute(base + collectionPath)}"></script>`;
}
