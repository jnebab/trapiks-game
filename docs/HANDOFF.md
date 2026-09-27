# Handoff

This is the state of Trapiks for the next session, and how to pick it up.

## Resume in one message

Start a new session on branch `claude/brave-pasteur-hlfvly` in the environment whose network access is set to all domains. Paste:

```
Continue Trapiks on branch claude/brave-pasteur-hlfvly. Read docs/HANDOFF.md first,
then CLAUDE.md, docs/plan.md and docs/milestones/M16.md. Do M16, then M17.
```

**First check:** `curl -sS -o /dev/null -w "%{http_code}" https://overpass-api.de/api/status` must print 200. A 403 means the network setting has not reached the session.

## Where things stand

| Area | State |
|---|---|
| M0–M15c | Done, reviewed, committed and pushed (last code commit `1625319`) |
| M16, the real Metro Manila map | Spec ready in `docs/milestones/M16.md`, not started. It was blocked by the network policy (overpass-api.de answered 403) |
| M17, city performance on the real map | Planned in `docs/plan.md` §6. No spec yet; write it after M16 with real-map numbers |
| `scripts/check.sh` | Passes on `1625319`: fmt, clippy, all Rust tests, wasm build, native = wasm hash check, typecheck, lint, format, vitest, comments, 18 Playwright specs |
| Default map | `synthetic` (`web/src/main.ts`, `DEFAULT_MAP`). M16 switches it to `metro-manila` |

The game is fully playable on the synthetic 30×30 city:
- **Screens:** title, challenge list, and three synthetic challenges with baseline, Evaluate and stars.
- **Sandbox:** demand slider, stats and the traffic layer on `T`.
- **Saves:** kept in localStorage, with Continue.
- **Edits:** delete road, lanes, direction, speed, junction control, signal timing, turn bans, flyovers, roundabouts and new connector roads, all with undo.
- **Look:** Trafficity-style rendering with shadows, signal pills, markings, buildings, and jeepneys and buses.

## How the work is run

- **Roles** (see `CLAUDE.md`):
  - The main session orchestrates. It writes each milestone spec and reviews every diff itself: it reads the code, runs `bash scripts/check.sh`, and looks at the screenshots.
  - The implementer (`.claude/agents/implementer.md`, Opus 5.5 at low effort) writes the code.
  - Fable 5.1 is no longer used for reviews, at the user's request.
- **Loop per milestone:**
  1. The implementer builds it from `docs/milestones/Mx.md` and leaves changes unstaged.
  2. The orchestrator reviews and runs the checks.
  3. Findings go back to the implementer.
  4. The orchestrator commits and pushes.
- **Lessons from this project:**
  - Run exactly one implementer at a time. Two concurrent implementers once overwrote each other's files.
  - Implementers must not stage or commit. Commit with explicit paths.
  - Tell implementers to run everything in the foreground and to end with a full report. One run ended early while it waited on a background job.
  - `test-results/` is wiped on every Playwright run, so screenshots must be copied to the scratchpad.
  - The user asked to be consulted on important decisions.
- **The Trafficity reference screenshots are not in the repo** (plan §4: local only). The old scratchpad copies do not carry over to a new session. Ask the user to upload them again if a visual review needs them.

## M16 notes

- `scripts/fetch-osm.sh` queries `https://overpass-api.de/api/interpreter` in four quadrants into `data/osm/q{1..4}.json`, which is gitignored.
- `scripts/build-map.sh` writes `web/public/maps/metro-manila.bin.gz`. Commit it with the ODbL `LICENSE` beside it.
- The map format is v4: v3 added the roundabout road flag, and v4 added area holes.
- The seven Metro Manila challenges are in `web/public/challenges/metro-manila.json`.
  - Their coordinates are unverified.
  - Their budgets are 8,000–12,000 pesos.
  - M16 checks each centre against the real junction and tunes `vehicles_per_hour`.
- Money is shown in Philippine pesos everywhere (`web/src/hud/money.ts`).

## Known gaps and decisions

**Performance** (full numbers in `docs/perf.md`):

| Measure | Now | Target | Status |
|---|---|---|---|
| City step, native (synthetic 120×120, ~23.8k vehicles) | 19.7 ms median | 12 ms | missed |
| City step, wasm | 37 ms | 25 ms | missed |
| Mean A* | 1.0 ms | 0.5 ms | missed |
| Region step | 1.2 ms | 2 ms | met |
| Deleting the busiest road | 45 ms max | 50 ms | met |

- Proposed M17 fixes, from the splits in `docs/perf.md`:
  - share each vehicle's leader and gap across the acceleration, decision and lane-change passes
  - a lane-change evaluation window
  - better A* bounds or route caching
  - patch-based rebuilds for edits that append roads
- The user chose to measure and optimize on the real map, as M17.

**Decisions made during the build:**
- **Weighted A\*:** `ROUTE_HEURISTIC_WEIGHT = 2.0`. Routes are about 7 % longer than optimal on average in congestion.
- **Ring entries** need room for two vehicles on the ring lane, which keeps small roundabouts from gridlocking.
- **Challenge budgets** were roughly doubled so a flyover (about ₱3,500) plus a few smaller fixes fit.

**Synthetic baselines after M14:**

| Challenge | Mean delay |
|---|---|
| Downtown | 240.5 s |
| Riverside | 221.4 s |
| Skyway Exit | 379.8 s (saturated, by design) |

**Headless Chromium** renders in software (1–15 FPS). Real frame rates need a GPU and have not been measured.

## Map of the code

| Path | Content |
|---|---|
| `crates/sim-core/src/` | `map/`, `network/`, `vehicle/`, `sim/`, `routing/`, `demand/`, `edit/`, `challenge/`, `render/` (geometry for the web renderer), `fixtures/` (test maps and scenarios) |
| `crates/sim-wasm/src/` | `Engine`, `MapHandle`, `ChallengeRunner`, typed-array geometry getters |
| `crates/mapgen/` | Overpass JSON to `.bin.gz`, plus `--synthetic CxR@S` |
| `web/src/app/` | game shell, router, screens, saves |
| `web/src/worker/` | sim loop, sessions, challenge runs |
| `web/src/render/` | tiles, layers, markings, shadows, buildings, vehicles, traffic layer |
| `web/src/edit/`, `web/src/hud/` | picking, tools, inspector, toolbar, chips |
| `examples/bench` | `cargo run --release -p trapiks-sim-core --example bench -- <map> [--region x,y,r] [--vph N] [--warmup N] [--ticks N] [--delete-road-at T] [--roundabout-at T] [--add-road-at T]` |
