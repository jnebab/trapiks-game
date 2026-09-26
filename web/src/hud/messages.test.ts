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

  it('names every new road rejection', () => {
    expect(errorMessage('InvalidLayer')).toBe('Elevation out of range');
    expect(errorMessage('EndpointIsolated')).toBe('Junction has no roads');
    expect(errorMessage('TooCloseToEnd')).toBe('Too close to the end of the road');
    expect(errorMessage('SameEndpoint')).toBe('Both ends are the same');
    expect(errorMessage('InvalidLength')).toBe('Road too short or too long');
    expect(errorMessage('AngleTooSharp')).toBe('Angle too sharp');
    expect(errorMessage('CrossesRoad')).toBe('Crosses another road');
  });
});
