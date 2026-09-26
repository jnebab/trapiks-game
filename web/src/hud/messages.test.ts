import { describe, expect, it } from 'vitest';
import { errorMessage } from './messages';

describe('errorMessage', () => {
  it('names every roundabout rejection', () => {
    expect(errorMessage('InvalidRadius')).toBe('Radius out of range');
    expect(errorMessage('AlreadyRoundabout')).toBe('Already a roundabout');
    expect(errorMessage('RoundaboutTooLarge')).toBe('Roads too short for this size');
    expect(errorMessage('LayerMismatch')).toBe('Roads are on different levels');
    expect(errorMessage('RoundaboutTooTight')).toBe('Roads too close together');
  });
});
