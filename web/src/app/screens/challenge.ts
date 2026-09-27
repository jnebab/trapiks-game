import type { Challenge } from '../../generated/Challenge';
import { el } from '../../hud/dom';
import type { EvaluationMessage } from '../../sim/protocol';
import { challengeMode } from '../challenge-list';
import type { GameContext, Screen } from '../screen';
import { createFirstHint } from './first-hint';
import { createObjective, progressText, type Objective } from './objective';
import { button, createTopBar, screenRoot } from './parts';
import { createResultModal } from './result';
import { loadSave, logOf } from './save-loading';

interface ChallengeParts {
  element: HTMLElement;
  slot: HTMLElement;
  objective: Objective;
  evaluate: HTMLButtonElement;
  progress: HTMLElement;
}

function start(ctx: GameContext, challenge: Challenge, fresh: boolean): void {
  const mode = challengeMode(challenge.id);
  if (fresh) {
    ctx.saves.clear(ctx.mapHash, mode);
  }
  const save = fresh ? undefined : loadSave(ctx, mode);
  ctx.startChallenge(challenge, logOf(save));
  ctx.setSaveTarget({ mode, seed: challenge.seed });
}

function nextRoute(ctx: GameContext, challenge: Challenge): void {
  ctx.challenges().then(
    (list) => {
      const index = list.findIndex((item) => item.id === challenge.id);
      const next = list[index + 1];
      ctx.navigate(
        next === undefined ? { name: 'challenges' } : { name: 'challenge', id: next.id },
      );
    },
    () => {
      ctx.navigate({ name: 'challenges' });
    },
  );
}

function buildParts(ctx: GameContext, challenge: Challenge, reset: () => void): ChallengeParts {
  const element = screenRoot('challenge-screen');
  const resetChip = button('Reset', 'chip', reset);
  resetChip.id = 'reset';
  const bar = createTopBar(ctx, {
    title: challenge.name,
    back: { name: 'challenges' },
    extras: [resetChip],
  });
  const objective = createObjective(challenge);
  const evaluate = button('Evaluate', 'chip primary evaluate', () => undefined);
  evaluate.id = 'evaluate';
  evaluate.disabled = true;
  const bottom = el('div', 'bottom-left');
  bottom.append(objective.element, evaluate);
  const progress = el('div', 'progress-overlay');
  progress.id = 'evaluating';
  progress.hidden = true;
  element.append(bar.element, bottom, progress);
  const hint = createFirstHint();
  if (hint !== undefined) {
    element.appendChild(hint);
  }
  return { element, slot: bar.slot, objective, evaluate, progress };
}

function setEvaluating(ctx: GameContext, parts: ChallengeParts, running: boolean): void {
  parts.progress.hidden = !running;
  parts.evaluate.disabled = running;
  ctx.mapScene()?.edit?.setLocked(running);
}

function showResult(ctx: GameContext, parts: ChallengeParts, challenge: Challenge) {
  return (message: EvaluationMessage): void => {
    setEvaluating(ctx, parts, false);
    ctx.saves.recordStars(ctx.mapHash, challengeMode(challenge.id), message.score.stars);
    const modal = createResultModal(message.score, {
      keepFixing: () => undefined,
      next: () => {
        nextRoute(ctx, challenge);
      },
    });
    parts.element.appendChild(modal);
  };
}

function runScreen(ctx: GameContext, challenge: Challenge, parts: ChallengeParts): Screen {
  parts.evaluate.addEventListener('click', () => {
    parts.progress.textContent = progressText('Evaluating', 0, 1);
    setEvaluating(ctx, parts, true);
    ctx.client.evaluate();
  });
  return {
    element: parts.element,
    editHost: { root: parts.element, slot: parts.slot },
    onReady: (ready) => {
      parts.progress.hidden = true;
      parts.evaluate.disabled = true;
      if (ready.region !== null && !ctx.cameraQuery) {
        ctx.flyToRegion(ready.region);
      }
    },
    onRunProgress: ({ phase, done, total }) => {
      if (phase === 'baseline') {
        parts.objective.measuring(done, total);
        return;
      }
      parts.progress.textContent = progressText('Evaluating', done, total);
    },
    onBaseline: ({ result }) => {
      parts.objective.measured(result);
      parts.evaluate.disabled = false;
    },
    onEvaluation: showResult(ctx, parts, challenge),
  };
}

interface Delegate {
  screen?: Screen;
  disposed: boolean;
}

function open(ctx: GameContext, id: string, host: HTMLElement, delegate: Delegate) {
  return (list: Challenge[]): void => {
    const challenge = list.find((item) => item.id === id);
    if (delegate.disposed) {
      return;
    }
    if (challenge === undefined) {
      ctx.navigate({ name: 'title' });
      return;
    }
    const parts = buildParts(ctx, challenge, () => {
      start(ctx, challenge, true);
    });
    host.replaceChildren(parts.element);
    delegate.screen = runScreen(ctx, challenge, parts);
    start(ctx, challenge, false);
  };
}

export function createChallengeScreen(ctx: GameContext, id: string): Screen {
  const element = screenRoot('challenge-host');
  const delegate: Delegate = { disposed: false };
  ctx.challenges().then(open(ctx, id, element, delegate), () => {
    ctx.toast('Challenges could not be loaded');
  });
  return delegateScreen(element, delegate);
}

function delegateScreen(element: HTMLElement, delegate: Delegate): Screen {
  return {
    element,
    dispose: () => {
      delegate.disposed = true;
    },
    get editHost() {
      return delegate.screen?.editHost;
    },
    onReady: (message) => delegate.screen?.onReady?.(message),
    onRunProgress: (message) => delegate.screen?.onRunProgress?.(message),
    onBaseline: (message) => delegate.screen?.onBaseline?.(message),
    onEvaluation: (message) => delegate.screen?.onEvaluation?.(message),
  };
}
