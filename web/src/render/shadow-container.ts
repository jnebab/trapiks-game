import { AlphaFilter, BlurFilter, Container } from 'pixi.js';
import { shadowBlurStrength, shadowNeedsUpdate } from './shadow-style';

const SHADOW_ALPHA = 0.13;
const BLUR_QUALITY = 3;

export interface ShadowContainer {
  container: Container;
  update: (visible: boolean, scale: number) => void;
}

export function createShadowContainer(): ShadowContainer {
  const container = new Container();
  const blur = new BlurFilter({ strength: shadowBlurStrength(1), quality: BLUR_QUALITY });
  container.filters = [blur, new AlphaFilter({ alpha: SHADOW_ALPHA })];
  let appliedScale = 1;
  const update = (visible: boolean, scale: number): void => {
    container.visible = visible;
    if (!shadowNeedsUpdate(appliedScale, scale)) {
      return;
    }
    appliedScale = scale;
    blur.strength = shadowBlurStrength(scale);
  };
  return { container, update };
}
