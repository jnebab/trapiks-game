# Handoff

This is the state of Trapiks for the next session, and how to pick it up.

## Resume in one message

Start a new session on branch `claude/trapiks-m16-m17-vu6jwe`. Paste:

```
Continue Trapiks on branch claude/trapiks-m16-m17-vu6jwe. Read docs/HANDOFF.md first,
then CLAUDE.md, docs/plan.md and docs/perf.md. Propose the next milestone.
```

## Where things stand

| Area | State |
|---|---|
| M0–M17 | Done, reviewed, committed and pushed to `claude/trapiks-m16-m17-vu6jwe` |
| Map | The real Metro Manila map is committed at `web/public/maps/metro-manila.bin.gz` (138,807 roads, 5.46 MB) and is the default map. `?map=synthetic` loads the test city |
| Challenges | Seven real chokepoints with 600–700 m regions. Only C-5–Kalayaan and Taft–Buendia have a known 1-star fix; see the M16 Outcome for the scoring gap |
| Performance | Native city step 10.4 ms and wasm 18 ms at 20k vehicles. Open items are listed in `docs/perf.md` |
| Deploy | GitHub Pages workflow (`.github/workflows/pages.yml`); Cloudflare Pages and Vercel in `docs/deploy.md` |
| `scripts/check.sh` | Passes, including 4 real-map Playwright specs (22 e2e in total) |

**Candidate next milestones:**
1. **Challenge scoring:** score per completed trip, or per vehicle spawned in the window, then re-tune so each challenge has a 1-star fix.
2. **Patch-based rebuilds for appending edits:** roundabouts and new roads take 270–320 ms on the real map.
3. **Faster routing:** mean A* is about 1 ms against 0.5 ms. Options are bidirectional search, contraction or hub labels.

**Refetching OSM.** `scripts/fetch-osm.sh` retries each of its 8 tiles up to 60 times and resumes from finished tiles, because overpass-api.de rejects most requests from the cloud egress. The environment must allow `overpass-api.de`.

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

## Decisions made during the build

- **Weighted A\*:** `ROUTE_HEURISTIC_WEIGHT = 2.0`, with a closed set since M17. Routes are about 10 % longer than optimal in congestion.
- **Demand reachability (M16):** origins must reach the largest SCC, and destinations must be reachable from it. After an edit, the update runs in the next step.
- **Centre check (M16):** a challenge centre passes when the degree ≥ 3 junctions within 150 m together carry the named roads.
- **Ring entries** need room for two vehicles on the ring lane, which keeps small roundabouts from gridlocking.
- **Challenge budgets** are ₱8,000–12,000, so a flyover (about ₱3,500) plus a few smaller fixes fit. Money is shown in pesos (`web/src/hud/money.ts`).
- **Map format v4:** v3 added the roundabout road flag, and v4 added area holes.

**Synthetic baselines:**

| Challenge | Mean delay |
|---|---|
| Downtown | 240.5 s |
| Riverside | 221.4 s |
| Skyway Exit | 379.8 s (saturated, by design) |

**Headless Chromium** renders in software. Real frame rates need a GPU and have not been measured.

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
| `examples/challenge_check` | `cargo run --release -p trapiks-sim-core --example challenge_check -- <map> [challenges.json] [--only ID] [--vph N] [--radius M] [--sites-only] [--no-probes] [--fix OUT]`: centre check, baseline, and probe fixes, alone and in pairs |
| `examples/route_diag` | `cargo run --release -p trapiks-sim-core --example route_diag -- <map> --vph N --warmup N`: A* variants against Dijkstra |
| `tests/real_map.rs` | `TRAPIKS_REAL_MAP=1 cargo test --release -p trapiks-sim-core --test real_map -- --ignored --nocapture` |
