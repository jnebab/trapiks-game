import type { Challenge } from '../generated/Challenge';
import { isChallengeList } from '../sim/game-values';

export async function loadChallenges(mapName: string): Promise<Challenge[]> {
  const url = new URL(`challenges/${mapName}.json`, document.baseURI).href;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Challenges not found for ${mapName}`);
  }
  const list: unknown = await response.json();
  if (!isChallengeList(list)) {
    throw new Error(`Invalid challenges for ${mapName}`);
  }
  return list;
}

const CHALLENGE_MODE = 'challenge:';

export function challengeMode(id: string): `challenge:${string}` {
  return `${CHALLENGE_MODE}${id}`;
}

export function challengeIdOf(mode: `challenge:${string}`): string {
  return mode.slice(CHALLENGE_MODE.length);
}
