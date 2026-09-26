import { describe, expect, it } from 'vitest';
import { isMainMessage, isWorkerMessage } from './protocol';

const challenge = {
  id: 'downtown',
  name: 'Downtown Box',
  blurb: 'Untangle the grid.',
  center: { Local: { x: 1500, y: 1500 } },
  radius_m: 900,
  seed: 1,
  vehicles_per_hour: 9000,
  budget: 8000,
  target_improvement: 0.2,
  star_steps: [0.1, 0.2],
  min_throughput_ratio: 0.95,
};

const run = {
  mean_delay_s: 42.3,
  throughput_per_hour: 5000,
  created: 100,
  arrivals: 90,
  unserved: 1,
  stranded: 0,
};

const score = {
  improvement: 0.25,
  throughput_ratio: 1,
  cost: 1200,
  passed: true,
  stars: 2,
  reasons: [],
};

describe('game messages from the main thread', () => {
  const log = [{ DeleteRoad: { road: 3 } }, 'Undo'];

  it('accepts sandbox and challenge starts with or without a log', () => {
    expect(isMainMessage({ type: 'startSandbox', session: 1, vehiclesPerHour: 3000 })).toBe(true);
    expect(isMainMessage({ type: 'startSandbox', session: 2, vehiclesPerHour: 0, log })).toBe(true);
    expect(isMainMessage({ type: 'startChallenge', session: 3, challenge })).toBe(true);
    expect(isMainMessage({ type: 'startChallenge', session: 3, challenge, log })).toBe(true);
  });

  it('rejects starts without a session or with a bad payload', () => {
    expect(isMainMessage({ type: 'startSandbox', vehiclesPerHour: 3000 })).toBe(false);
    expect(isMainMessage({ type: 'startSandbox', session: 1, vehiclesPerHour: -5 })).toBe(false);
    const badLog = { type: 'startSandbox', session: 1, vehiclesPerHour: 1, log: ['Redo'] };
    expect(isMainMessage(badLog)).toBe(false);
    const badChallenge = { ...challenge, center: { Elsewhere: {} } };
    expect(isMainMessage({ type: 'startChallenge', session: 1, challenge: badChallenge })).toBe(
      false,
    );
  });

  it('accepts evaluate and demand changes', () => {
    expect(isMainMessage({ type: 'evaluate' })).toBe(true);
    expect(isMainMessage({ type: 'setDemand', vehiclesPerHour: 20000 })).toBe(true);
    expect(isMainMessage({ type: 'setDemand', vehiclesPerHour: '20000' })).toBe(false);
  });
});

describe('game messages from the worker', () => {
  it('accepts run progress, baseline, evaluation and notice', () => {
    const progress = { type: 'runProgress', session: 1, phase: 'baseline', done: 32, total: 9000 };
    expect(isWorkerMessage(progress)).toBe(true);
    expect(isWorkerMessage({ ...progress, phase: 'evaluate' })).toBe(true);
    expect(isWorkerMessage({ type: 'baseline', session: 1, result: run })).toBe(true);
    expect(isWorkerMessage({ type: 'evaluation', session: 1, result: run, score })).toBe(true);
    expect(
      isWorkerMessage({ type: 'notice', session: 1, text: 'Save could not be restored' }),
    ).toBe(true);
  });

  it('rejects malformed game messages', () => {
    const progress = { type: 'runProgress', session: 1, phase: 'warmup', done: 0, total: 1 };
    expect(isWorkerMessage(progress)).toBe(false);
    expect(isWorkerMessage({ type: 'baseline', session: 1, result: { mean_delay_s: 1 } })).toBe(
      false,
    );
    const badScore = { ...score, reasons: ['TooSlow'] };
    expect(isWorkerMessage({ type: 'evaluation', session: 1, result: run, score: badScore })).toBe(
      false,
    );
    expect(isWorkerMessage({ type: 'notice', session: 1 })).toBe(false);
  });
});

describe('session stamps', () => {
  it('rejects every worker message without a numeric session', () => {
    const messages = [
      { type: 'error', message: 'boom' },
      { type: 'signals', states: new Uint8Array(1) },
      { type: 'quoteResult', id: 1, result: { Ok: 1 } },
      { type: 'inspection', id: 1, target: null },
      { type: 'runProgress', phase: 'baseline', done: 0, total: 1 },
      { type: 'baseline', result: run },
      { type: 'evaluation', result: run, score },
      { type: 'notice', text: 'x' },
      {
        type: 'commandResults',
        results: [],
        delta: null,
        budget: { limit: null, spent: 0 },
        log: [],
      },
    ];
    for (const message of messages) {
      expect(isWorkerMessage({ ...message, session: 4 })).toBe(true);
      expect(isWorkerMessage(message)).toBe(false);
      expect(isWorkerMessage({ ...message, session: '4' })).toBe(false);
    }
  });

  it('requires the command log on command results', () => {
    const base = { type: 'commandResults', session: 0, results: [], delta: null };
    const budget = { limit: 10, spent: 0 };
    expect(
      isWorkerMessage({ ...base, budget, log: [{ SetDemand: { vehicles_per_hour: 1 } }] }),
    ).toBe(true);
    expect(isWorkerMessage({ ...base, budget })).toBe(false);
  });
});
