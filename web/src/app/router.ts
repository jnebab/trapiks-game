export type Route =
  | { name: 'title' }
  | { name: 'challenges' }
  | { name: 'challenge'; id: string }
  | { name: 'sandbox' }
  | { name: 'guide' };

const CHALLENGE_PREFIX = '#/challenge/';
const CHALLENGE_ID = /^[a-z0-9-]+$/;

function challengeRoute(hash: string): Route | undefined {
  if (!hash.startsWith(CHALLENGE_PREFIX)) {
    return undefined;
  }
  const id = decodeURIComponent(hash.slice(CHALLENGE_PREFIX.length));
  return CHALLENGE_ID.test(id) ? { name: 'challenge', id } : undefined;
}

export function parseRoute(hash: string): Route {
  if (hash === '#/challenges') {
    return { name: 'challenges' };
  }
  if (hash === '#/sandbox') {
    return { name: 'sandbox' };
  }
  if (hash === '#/guide') {
    return { name: 'guide' };
  }
  return challengeRoute(hash) ?? { name: 'title' };
}

export function routeHash(route: Route): string {
  switch (route.name) {
    case 'title':
      return '#/';
    case 'challenges':
      return '#/challenges';
    case 'sandbox':
      return '#/sandbox';
    case 'guide':
      return '#/guide';
    case 'challenge':
      return `${CHALLENGE_PREFIX}${encodeURIComponent(route.id)}`;
  }
}

export function navigate(route: Route): void {
  window.location.hash = routeHash(route);
}

export function onRouteChange(render: (route: Route) => void): void {
  window.addEventListener('hashchange', () => {
    render(parseRoute(window.location.hash));
  });
}

export function currentRoute(): Route {
  return parseRoute(window.location.hash);
}

export function hasCameraQuery(search: string): boolean {
  const params = new URLSearchParams(search);
  return ['cx', 'cy', 'z'].every((name) => params.get(name) !== null);
}
