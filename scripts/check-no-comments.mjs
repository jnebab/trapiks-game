import { readdirSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const webRoot = join(repoRoot, 'web');
const require = createRequire(join(webRoot, 'package.json'));
const ts = require('typescript');

const excludedTsDirs = [join(webRoot, 'src', 'generated'), join(webRoot, 'src', 'wasm')];

function walk(dir, extension, excluded = []) {
  if (excluded.includes(dir)) {
    return [];
  }
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      return walk(path, extension, excluded);
    }
    return entry.name.endsWith(extension) ? [path] : [];
  });
}

function rootConfigFiles() {
  return readdirSync(webRoot, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.ts'))
    .map((entry) => join(webRoot, entry.name));
}

function lineOf(text, offset) {
  let line = 1;
  for (let i = 0; i < offset; i += 1) {
    if (text[i] === '\n') {
      line += 1;
    }
  }
  return line;
}

function addCommentRanges(offsets, ranges) {
  for (const range of ranges ?? []) {
    offsets.add(range.pos);
  }
}

export function tsCommentOffsets(text) {
  const source = ts.createSourceFile('file.ts', text, ts.ScriptTarget.Latest, true);
  const offsets = new Set();
  const visit = (node) => {
    addCommentRanges(offsets, ts.getLeadingCommentRanges(text, node.getFullStart()));
    addCommentRanges(offsets, ts.getTrailingCommentRanges(text, node.getEnd()));
    node.getChildren(source).forEach(visit);
  };
  visit(source);
  return [...offsets].sort((x, y) => x - y);
}

function skipQuoted(text, start, quote) {
  let i = start + 1;
  while (i < text.length && text[i] !== quote) {
    i += text[i] === '\\' ? 2 : 1;
  }
  return i + 1;
}

function countHashes(text, start) {
  let hashes = 0;
  while (text[start + hashes] === '#') {
    hashes += 1;
  }
  return hashes;
}

function skipRawString(text, start) {
  const hashes = countHashes(text, start + 1);
  const terminator = '"' + '#'.repeat(hashes);
  const end = text.indexOf(terminator, start + hashes + 2);
  return end === -1 ? text.length : end + terminator.length;
}

function skipCharOrLifetime(text, start) {
  if (text[start + 1] === '\\') {
    return skipQuoted(text, start, "'");
  }
  if (text[start + 2] === "'") {
    return start + 3;
  }
  return start + 1;
}

function isRawStringStart(text, i) {
  const prefix = text[i] === 'b' ? i + 1 : i;
  if (text[prefix] !== 'r') {
    return -1;
  }
  const hashes = countHashes(text, prefix + 1);
  return text[prefix + 1 + hashes] === '"' ? prefix : -1;
}

function isIdentChar(char) {
  return char !== undefined && /[A-Za-z0-9_]/.test(char);
}

function skipLiteral(text, i) {
  const rawStart = isIdentChar(text[i - 1]) ? -1 : isRawStringStart(text, i);
  if (rawStart !== -1) {
    return skipRawString(text, rawStart);
  }
  if (text[i] === '"') {
    return skipQuoted(text, i, '"');
  }
  if (text[i] === "'") {
    return skipCharOrLifetime(text, i);
  }
  return -1;
}

function commentEnd(text, i) {
  if (text[i + 1] === '/') {
    const end = text.indexOf('\n', i);
    return end === -1 ? text.length : end;
  }
  const end = text.indexOf('*/', i + 2);
  return end === -1 ? text.length : end + 2;
}

function nextNonBlankLine(text, from) {
  const lines = text.slice(from).split('\n').slice(1);
  return lines.find((line) => line.trim() !== '') ?? '';
}

function isSafetyComment(text, i) {
  return text.startsWith('// SAFETY:', i) && nextNonBlankLine(text, i).includes('unsafe');
}

export function rustCommentOffsets(text) {
  const offsets = [];
  let i = 0;
  while (i < text.length) {
    const literalEnd = skipLiteral(text, i);
    if (literalEnd !== -1) {
      i = literalEnd;
      continue;
    }
    const isComment = text[i] === '/' && (text[i + 1] === '/' || text[i + 1] === '*');
    if (!isComment) {
      i += 1;
      continue;
    }
    if (!isSafetyComment(text, i)) {
      offsets.push(i);
    }
    i = commentEnd(text, i);
  }
  return offsets;
}

function report(files, findOffsets) {
  return files.flatMap((file) => {
    const text = readFileSync(file, 'utf8');
    return findOffsets(text).map((offset) => `${relative(repoRoot, file)}:${lineOf(text, offset)}`);
  });
}

function main() {
  const tsFiles = [
    ...walk(join(webRoot, 'src'), '.ts', excludedTsDirs),
    ...walk(join(webRoot, 'e2e'), '.ts'),
    ...rootConfigFiles(),
    ...walk(join(repoRoot, 'scripts'), '.mjs'),
  ];
  const rustFiles = walk(join(repoRoot, 'crates'), '.rs');
  const findings = [...report(tsFiles, tsCommentOffsets), ...report(rustFiles, rustCommentOffsets)];
  for (const finding of findings) {
    console.log(finding);
  }
  process.exit(findings.length > 0 ? 1 : 0);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
