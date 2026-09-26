import type { EditCommand } from '../../generated/EditCommand';
import type { SaveMode, SaveRecord } from '../saves';
import type { GameContext } from '../screen';

export const OTHER_MAP_TOAST = 'Save is for a different map';

export function loadSave(ctx: GameContext, mode: SaveMode): SaveRecord | undefined {
  const lookup = ctx.saves.read(ctx.mapHash, mode);
  if (lookup.kind === 'otherMap') {
    ctx.toast(OTHER_MAP_TOAST);
  }
  return lookup.kind === 'found' ? lookup.save : undefined;
}

export function logOf(save: SaveRecord | undefined): EditCommand[] | undefined {
  return save === undefined || save.log.length === 0 ? undefined : save.log;
}
