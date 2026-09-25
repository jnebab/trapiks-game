import assert from 'node:assert/strict';
import { test } from 'node:test';
import { rustCommentOffsets, tsCommentOffsets } from './check-no-comments.mjs';

const tsCount = (text) => tsCommentOffsets(text).length;
const rustCount = (text) => rustCommentOffsets(text).length;

test('ts ignores comment markers inside strings', () => {
  assert.equal(tsCount("const a = '// no'; const b = \"/* no */\";\n"), 0);
});

test('ts ignores markers in template literals', () => {
  assert.equal(tsCount('const a = `// no /* no */`;\n'), 0);
  assert.equal(tsCount('const a = `// ${1} /* no */`;\n'), 0);
});

test('ts finds a comment after a template substitution', () => {
  assert.equal(tsCount('const a = `${1}x`;\n// yes\nconst b = 2;\n'), 1);
});

test('ts ignores regex literals', () => {
  assert.equal(tsCount('const r = /https?:\\/\\//;\n'), 0);
});

test('ts finds comments inside empty call args', () => {
  assert.equal(tsCount('foo(/* a */);\n'), 1);
});

test('ts finds a trailing comment at end of file', () => {
  assert.equal(tsCount('const a = 1;\n// end'), 1);
});

test('ts finds triple-slash directives', () => {
  assert.equal(tsCount('/// <reference types="vite/client" />\nexport {};\n'), 1);
});

test('rust raw identifier does not hide a comment', () => {
  assert.equal(rustCount('let r#type = 1;\n// comment\n'), 1);
});

test('rust ignores markers in raw and byte strings', () => {
  assert.equal(rustCount('let a = r#"..//.."#; let b = br"/*"; let c = b"//";\n'), 0);
});

test('rust handles lifetimes and char literals', () => {
  assert.equal(rustCount("fn f<'a>(x: &'a str) { let q = '\\''; let s = '/'; }\n// c\n"), 1);
});

test('rust finds block comments', () => {
  assert.equal(rustCount('let a = 1; /* block */\n'), 1);
});

test('rust allows SAFETY only before unsafe', () => {
  assert.equal(rustCount('// SAFETY: fine\n\nunsafe { f() }\n'), 0);
  assert.equal(rustCount('// SAFETY: not fine\nlet a = 1;\n'), 1);
});
