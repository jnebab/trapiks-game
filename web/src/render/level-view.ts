export type LevelView = 'all' | 'see-through' | 'ground' | 'elevated';

export const LEVEL_VIEWS: readonly LevelView[] = ['all', 'see-through', 'ground', 'elevated'];

export interface LevelStyle {
  alpha: number;
  visible: boolean;
  shadow: boolean;
}

const TUNNEL_ALPHA = 0.4;
const SEE_THROUGH_ALPHA = 0.35;
const FADED_ALPHA = 0.25;
const PREFERRED_RANK = 100;

function baseAlpha(layer: number): number {
  return layer < 0 ? TUNNEL_ALPHA : 1;
}

function elevatedStyle(view: LevelView): LevelStyle {
  if (view === 'see-through') {
    return { alpha: SEE_THROUGH_ALPHA, visible: true, shadow: false };
  }
  return { alpha: 1, visible: view !== 'ground', shadow: view === 'all' || view === 'elevated' };
}

export function levelStyle(view: LevelView, layer: number): LevelStyle {
  if (layer >= 1) {
    return elevatedStyle(view);
  }
  const fade = view === 'elevated' ? FADED_ALPHA : 1;
  return { alpha: baseAlpha(layer) * fade, visible: true, shadow: false };
}

export function buildingsAlpha(view: LevelView): number {
  return view === 'elevated' ? FADED_ALPHA : 1;
}

export function pickRank(view: LevelView, layer: number): number | undefined {
  const elevated = layer >= 1;
  if (view === 'ground' && elevated) {
    return undefined;
  }
  if (view === 'see-through' && !elevated) {
    return layer + PREFERRED_RANK;
  }
  if (view === 'elevated' && elevated) {
    return layer + PREFERRED_RANK;
  }
  return layer;
}

export function isLevelShown(view: LevelView, layer: number): boolean {
  return levelStyle(view, layer).visible;
}

export function nextLevelView(view: LevelView): LevelView {
  const index = LEVEL_VIEWS.indexOf(view);
  return LEVEL_VIEWS[(index + 1) % LEVEL_VIEWS.length] ?? 'all';
}

export function parseLevelView(value: string | null): LevelView {
  return LEVEL_VIEWS.find((view) => view === value) ?? 'all';
}
