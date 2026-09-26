import { describe, expect, it } from 'vitest';
import { hasCameraQuery, parseRoute, routeHash } from './router';

describe('parseRoute', () => {
  it('parses every known screen', () => {
    expect(parseRoute('#/')).toEqual({ name: 'title' });
    expect(parseRoute('')).toEqual({ name: 'title' });
    expect(parseRoute('#/challenges')).toEqual({ name: 'challenges' });
    expect(parseRoute('#/challenge/downtown')).toEqual({ name: 'challenge', id: 'downtown' });
    expect(parseRoute('#/sandbox')).toEqual({ name: 'sandbox' });
  });

  it('sends unknown routes to the title', () => {
    expect(parseRoute('#/nowhere')).toEqual({ name: 'title' });
    expect(parseRoute('#/challenge/')).toEqual({ name: 'title' });
    expect(parseRoute('#/challenge/Bad Id')).toEqual({ name: 'title' });
  });

  it('round trips through routeHash', () => {
    const routes = [
      { name: 'title' },
      { name: 'challenges' },
      { name: 'challenge', id: 'skyway-exit' },
      { name: 'sandbox' },
    ] as const;
    for (const route of routes) {
      expect(parseRoute(routeHash(route))).toEqual(route);
    }
  });
});

describe('hasCameraQuery', () => {
  it('needs cx, cy and z together', () => {
    expect(hasCameraQuery('?map=synthetic&cx=1&cy=2&z=3')).toBe(true);
    expect(hasCameraQuery('?cx=1&cy=2')).toBe(false);
    expect(hasCameraQuery('')).toBe(false);
  });
});
