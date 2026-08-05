'use strict';

// Development-only inventory helper. It executes an upstream test file while
// proxying the built-in path module, then emits every expanded public call.
const Module = require('module');
const fs = require('fs');
const nativeAssert = require('assert');
const nativePath = require('path');

const target = process.env.NODE_PATH_CAPTURE_TARGET;
const sourceFile = process.env.NODE_PATH_CAPTURE_SOURCE;
const sourceRoot = process.env.NODE_PATH_CAPTURE_ROOT;
const outputFile = process.env.NODE_PATH_CAPTURE_OUT;
const forcedPlatform = process.env.NODE_PATH_CAPTURE_FORCE_PLATFORM;
const tolerateAssertions = process.env.NODE_PATH_CAPTURE_TOLERATE_ASSERTIONS === '1';
const calls = [];
let sequence = 0;
let lastResult;

function relevantCall() {
  return relevantFrame() !== undefined;
}

function relevantFrame() {
  if (!target) return undefined;
  const firstExternalFrame = new Error().stack
    .split('\n')
    .slice(1)
    .find((frame) => !frame.includes(__filename));
  return firstExternalFrame?.includes(target) ? firstExternalFrame : undefined;
}

function sourceLocation() {
  const frame = relevantFrame();
  const match = frame?.match(/:(\d+):(\d+)\)?$/);
  return match ? { source_line: Number(match[1]), source_column: Number(match[2]) } : {};
}

function encode(value, seen = new WeakSet()) {
  if (value === undefined) return { $type: 'undefined' };
  if (typeof value === 'bigint') return { $type: 'bigint', value: String(value) };
  if (typeof value === 'function') return { $type: 'function', name: value.name };
  if (typeof value === 'symbol') return { $type: 'symbol', value: String(value) };
  if (Number.isNaN(value)) return { $type: 'number', value: 'NaN' };
  if (typeof value === 'string' && sourceRoot) {
    const windowsRoot = sourceRoot.replaceAll('/', '\\');
    return value
      .replaceAll(sourceRoot, '/node-source')
      .replaceAll(sourceRoot.toLowerCase(), '/node-source')
      .replaceAll(windowsRoot, '\\node-source')
      .replaceAll(windowsRoot.toLowerCase(), '\\node-source');
  }
  if (value && typeof value === 'object') {
    for (const [namespace, proxy] of Object.entries(proxies)) {
      if (value === proxy) return { $type: 'path-namespace', namespace };
    }
    if (seen.has(value)) return { $type: 'circular' };
    seen.add(value);
    if (Array.isArray(value)) return value.map((item) => encode(item, seen));
    const encoded = {};
    for (const key of Object.keys(value)) encoded[key] = encode(value[key], seen);
    return encoded;
  }
  return value;
}

const publicMethods = new Set([
  'resolve', 'normalize', 'isAbsolute', 'join', 'relative',
  'toNamespacedPath', 'dirname', 'basename', 'extname',
  'format', 'parse', 'matchesGlob',
]);

const proxies = {};
const wrappers = new Map();

function proxyFor(namespace, object) {
  if (proxies[namespace]) return proxies[namespace];
  const proxy = new Proxy(object, {
    get(targetObject, property, receiver) {
      if (property === 'posix') return proxyFor('posix', nativePath.posix);
      if (property === 'win32') return proxyFor('win32', nativePath.win32);
      const value = Reflect.get(targetObject, property, receiver);
      if ((property === 'sep' || property === 'delimiter') && relevantCall()) {
        calls.push({
          sequence: sequence++, namespace, operation: property,
          arguments: [], result: encode(value), ...sourceLocation(),
        });
      }
      if (typeof property !== 'string' || !publicMethods.has(property) || typeof value !== 'function') {
        return value;
      }
      const key = `${namespace}:${property}`;
      if (wrappers.has(key)) return wrappers.get(key);
      const wrapper = function(...args) {
        const capture = relevantCall();
        try {
          const result = Reflect.apply(value, targetObject, args);
          if (capture) {
            const call = {
              sequence: sequence++, namespace, operation: property,
              arguments: encode(args), result: encode(result), ...sourceLocation(),
            };
            calls.push(call);
            lastResult = { value: result, call };
          }
          return result;
        } catch (error) {
          if (capture) {
            calls.push({
              sequence: sequence++, namespace, operation: property,
              arguments: encode(args),
              error: { name: error.name, code: error.code ?? null, message: error.message },
              ...sourceLocation(),
            });
          }
          throw error;
        }
      };
      wrappers.set(key, wrapper);
      return wrapper;
    },
  });
  proxies[namespace] = proxy;
  return proxy;
}

const defaultNamespace = forcedPlatform ?? (process.platform === 'win32' ? 'win32' : 'posix');
const defaultProxy = proxyFor(defaultNamespace, nativePath[defaultNamespace]);
const originalLoad = Module._load;
const commonWrappers = new Map();
const assertProxy = new Proxy(nativeAssert, {
  get(assert, property, receiver) {
    const value = Reflect.get(assert, property, receiver);
    if (!tolerateAssertions || property !== 'strictEqual') return value;
    return function(actual, expected, message) {
      const correspondsToLastResult = lastResult && (
        actual === lastResult.value ||
        (typeof actual === 'string' && typeof lastResult.value === 'string' &&
         actual === lastResult.value.toLowerCase())
      );
      if (relevantCall() && correspondsToLastResult) {
        lastResult.call.asserted_expected = encode(expected);
        lastResult.call.comparator = 'strict-equal';
      }
      try {
        return Reflect.apply(value, assert, [actual, expected, message]);
      } catch (error) {
        if (error?.code === 'ERR_ASSERTION') return;
        throw error;
      }
    };
  },
});
Module._load = function(request, parent, isMain) {
  if (request === 'assert' && parent?.filename === target) return assertProxy;
  if (request === 'path' || request === 'node:path') return defaultProxy;
  if (request === 'path/posix' || request === 'node:path/posix') {
    return proxyFor('posix', nativePath.posix);
  }
  if (request === 'path/win32' || request === 'node:path/win32') {
    return proxyFor('win32', nativePath.win32);
  }
  const loaded = Reflect.apply(originalLoad, this, [request, parent, isMain]);
  if (forcedPlatform && request === '../common' && parent?.filename === target) {
    if (!commonWrappers.has(loaded)) {
      commonWrappers.set(loaded, new Proxy(loaded, {
        get(common, property, receiver) {
          if (property === 'isWindows') return forcedPlatform === 'win32';
          if (property === 'skip' && forcedPlatform === 'win32') return () => {};
          return Reflect.get(common, property, receiver);
        },
      }));
    }
    return commonWrappers.get(loaded);
  }
  return loaded;
};

process.on('exit', () => {
  const contents = JSON.stringify({ source_file: sourceFile, platform: defaultNamespace, cases: calls });
  if (outputFile) fs.writeFileSync(outputFile, contents);
  else process.stdout.write(contents);
});
