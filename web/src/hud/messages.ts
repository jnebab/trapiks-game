import type { EditError } from '../generated/EditError';

const MESSAGES: Record<EditError, string> = {
  RoadNotFound: 'Road not found',
  RoadDeleted: 'Road already removed',
  NodeNotFound: 'Junction not found',
  OutsideRegion: 'Outside the challenge area',
  InvalidLanes: 'Invalid lane count',
  InvalidSpeed: 'Invalid speed limit',
  NotAJunction: 'Not a junction',
  NotSignalized: 'Junction has no signals',
  InvalidTiming: 'Invalid signal timing',
  TurnNotFound: 'Turn not found',
  NoChange: 'Nothing changed',
  InsufficientBudget: 'Not enough budget',
  NothingToUndo: 'Nothing to undo',
  DemandLocked: 'Demand is locked',
  InvalidDemand: 'Invalid demand',
  RoadNotIncident: 'Road does not meet this junction',
  FlyoverNotStraight: 'Flyover needs a straight crossing',
  FlyoverTooShort: 'Roads too short for a flyover',
  InvalidRadius: 'Radius out of range',
  AlreadyRoundabout: 'Already a roundabout',
  RoundaboutTooLarge: 'Roads too short for this size',
  LayerMismatch: 'Roads are on different levels',
  RoundaboutTooTight: 'Roads too close together',
  InvalidLayer: 'Elevation out of range',
  EndpointIsolated: 'Junction has no roads',
  TooCloseToEnd: 'Too close to the end of the road',
  SameEndpoint: 'Both ends are the same',
  InvalidLength: 'Road too short or too long',
  AngleTooSharp: 'Angle too sharp',
  CrossesRoad: 'Crosses another road',
};

export function errorMessage(error: EditError): string {
  return MESSAGES[error];
}
