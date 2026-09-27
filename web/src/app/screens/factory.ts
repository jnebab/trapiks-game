import type { Route } from '../router';
import type { GameContext, Screen } from '../screen';
import { createChallengeScreen } from './challenge';
import { createChallengesScreen } from './challenges';
import { createGuideScreen } from './guide';
import { createSandboxScreen } from './sandbox';
import { createTitleScreen } from './title';

export function createScreen(ctx: GameContext, route: Route): Screen {
  switch (route.name) {
    case 'title':
      return createTitleScreen(ctx);
    case 'challenges':
      return createChallengesScreen(ctx);
    case 'challenge':
      return createChallengeScreen(ctx, route.id);
    case 'sandbox':
      return createSandboxScreen(ctx);
    case 'guide':
      return createGuideScreen(ctx);
  }
}
