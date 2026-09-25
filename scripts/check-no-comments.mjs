import { readdirSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

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

function isCommentKind(kind) {
  return (
    kind === ts.SyntaxKind.SingleLineCommentTrivia || kind === ts.SyntaxKind.MultiLineCommentTrivia
  );
}

function tsCommentOffsets(text) {
  const scanner = ts.createScanner(ts.ScriptTarget.Latest, false, ts.LanguageVariant.Standard, text);
  const offsets = [];
  let kind = scanner.scan();
  while (kind !== ts.SyntaxKind.EndOfFileToken) {
    if (isCommentKind(kind)) {
      offsets.push(scanner.getTokenStart());
    }
    kind = scanner.scan();
  }
  return offsets;
}

function skipQuoted(text, start, quote) {
  let i = start + 1;
  while (i < text.length && text[i] !== quote) {
    i += text[i] === '\\' ? 2 : 1;
  }
  return i + 1;
}

function skipRawString(text, start) {
  let i = start + 1;
  let hashes = 0;
  while (text[i] === '#') {
    hashes += 1;
    i += 1;
  }
  const terminator = '"' + '#'.repeat(hashes);
  const end = text.indexOf(terminator, i + 1);
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
  const next = text[prefix + 1];
  return next === '"' || next === '#' ? prefix : -1;
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

function isSafetyComment(text, i) {
  return text.startsWith('// SAFETY:', i);
}

function rustCommentOffsets(text) {
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

const tsFiles = [...walk(join(webRoot, 'src'), '.ts', excludedTsDirs), ...rootConfigFiles()];
const rustFiles = walk(join(repoRoot, 'crates'), '.rs');
const findings = [...report(tsFiles, tsCommentOffsets), ...report(rustFiles, rustCommentOffsets)];

for (const finding of findings) {
  console.log(finding);
}
process.exit(findings.length > 0 ? 1 : 0);
