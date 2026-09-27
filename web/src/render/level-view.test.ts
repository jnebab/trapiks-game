import { describe, expect, it } from 'vitest';
import { buildingsAlpha, levelStyle, nextLevelView, parseLevelView, pickRank } from './level-view';

describe('levelStyle', () => {
  it('leaves every layer as is in the all view', () => {
    expect(levelStyle('all', 0)).toEqual({ alpha: 1, visible: true, shadow: false });
    expect(levelStyle('all', 2)).toEqual({ alpha: 1, visible: true, shadow: true });
    expect(levelStyle('all', -1).alpha).toBeCloseTo(0.4);
  });

  it('makes decks translucent without shadows in see-through', () => {
    expect(levelStyle('see-through', 1)).toEqual({ alpha: 0.35, visible: true, shadow: false });
    expect(levelStyle('see-through', 0).alpha).toBe(1);
  });

  it('hides decks in the ground view', () => {
    expect(levelStyle('ground', 1).visible).toBe(false);
    expect(levelStyle('ground', 0).visible).toBe(true);
  });

  it('fades ground, tunnels and buildings in the elevated view', () => {
    expect(levelStyle('elevated', 0).alpha).toBeCloseTo(0.25);
    expect(levelStyle('elevated', -2).alpha).toBeCloseTo(0.1);
    expect(levelStyle('elevated', 3)).toEqual({ alpha: 1, visible: true, shadow: true });
    expect(buildingsAlpha('elevated')).toBeCloseTo(0.25);
    expect(buildingsAlpha('all')).toBe(1);
  });
});

describe('pickRank', () => {
  it('prefers the highest layer in the all view', () => {
    expect(pickRank('all', 1)).toBeGreaterThan(pickRank('all', 0) ?? Infinity);
  });

  it('prefers ground in see-through', () => {
    expect(pickRank('see-through', 0)).toBeGreaterThan(pickRank('see-through', 2) ?? Infinity);
  });

  it('ignores decks in the ground view', () => {
    expect(pickRank('ground', 1)).toBeUndefined();
  });

  it('prefers decks in the elevated view', () => {
    expect(pickRank('elevated', 1)).toBeGreaterThan(pickRank('elevated', 0) ?? Infinity);
  });
});

describe('cycling', () => {
  it('cycles through every view and parses stored values', () => {
    expect(nextLevelView('all')).toBe('see-through');
    expect(nextLevelView('elevated')).toBe('all');
    expect(parseLevelView('ground')).toBe('ground');
    expect(parseLevelView('bogus')).toBe('all');
    expect(parseLevelView(null)).toBe('all');
  });
});
