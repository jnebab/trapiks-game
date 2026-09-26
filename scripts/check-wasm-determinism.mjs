import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const targetDir = join(root, 'target', 'wasm-fixtures');
const pkgDir = join(targetDir, 'pkg');
const wasmFile = join(targetDir, 'wasm32-unknown-unknown', 'release', 'trapiks_sim_wasm.wasm');
const scenarios = ['four_way_5000', 'grid_city_demand_1000'];

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: 'inherit' });
}

function build() {
  run('bash', ['-c', 'source scripts/lib/wasm-bindgen-version.sh && check_wasm_bindgen_version']);
  run('cargo', [
    'build',
    '-p',
    'trapiks-sim-wasm',
    '--target',
    'wasm32-unknown-unknown',
    '--release',
    '--features',
    'fixtures',
    '--target-dir',
    targetDir,
  ]);
  run('wasm-bindgen', ['--target', 'nodejs', '--out-dir', pkgDir, wasmFile]);
}

function check(wasm, name) {
  const [computed, expected] = wasm.runScenario(name).split(' ');
  const ok = computed === expected;
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${name}: wasm ${computed} native ${expected}`);
  return ok;
}

build();
const wasm = createRequire(import.meta.url)(join(pkgDir, 'trapiks_sim_wasm.js'));
const results = scenarios.map((name) => check(wasm, name));
process.exit(results.every(Boolean) ? 0 : 1);
