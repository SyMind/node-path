// Test-only line-delimited adapter for a pinned Node.js executable.
// Each input line is { id, namespace, operation, arguments } and each output
// line preserves the id with either { ok: true, result } or a normalized error.

import path from 'node:path';
import readline from 'node:readline';

const methods = new Map([
  ['resolve', 'resolve'],
  ['normalize', 'normalize'],
  ['is-absolute', 'isAbsolute'],
  ['isAbsolute', 'isAbsolute'],
  ['join', 'join'],
  ['relative', 'relative'],
  ['to-namespaced-path', 'toNamespacedPath'],
  ['toNamespacedPath', 'toNamespacedPath'],
  ['make-long', '_makeLong'],
  ['dirname', 'dirname'],
  ['basename', 'basename'],
  ['extname', 'extname'],
  ['format', 'format'],
  ['parse', 'parse'],
  ['matches-glob', 'matchesGlob'],
  ['matchesGlob', 'matchesGlob'],
]);

function namespaceFor(value) {
  if (value === 'posix') return path.posix;
  if (value === 'win32') return path.win32;
  if (value === 'host-default') return path;
  throw new TypeError(`unknown namespace: ${value}`);
}

function evaluate(request) {
  const namespace = namespaceFor(request.namespace);
  if (request.operation === 'sep' || request.operation === 'delimiter') {
    return namespace[request.operation];
  }
  const method = methods.get(request.operation);
  if (!method || typeof namespace[method] !== 'function') {
    throw new TypeError(`unknown operation: ${request.operation}`);
  }
  const args = Array.isArray(request.arguments) ? request.arguments : [];
  return Reflect.apply(namespace[method], namespace, args);
}

const lines = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of lines) {
  if (!line.trim()) continue;
  let request;
  try {
    request = JSON.parse(line);
    const result = evaluate(request);
    process.stdout.write(`${JSON.stringify({ id: request.id ?? null, ok: true, result })}\n`);
  } catch (error) {
    process.stdout.write(`${JSON.stringify({
      id: request?.id ?? null,
      ok: false,
      error: {
        name: error?.name ?? 'Error',
        code: error?.code ?? null,
        message: error?.message ?? String(error),
      },
    })}\n`);
  }
}
