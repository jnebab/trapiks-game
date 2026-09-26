import type { Challenge } from '../generated/Challenge';
import type { EditCommand } from '../generated/EditCommand';
import type { CameraState } from '../render/camera-input';
import type { SimClient } from '../sim/client';
import type {
  BaselineMessage,
  CommandResultsMessage,
  EvaluationMessage,
  ReadyMessage,
  Region,
  RunProgressMessage,
  StatsMessage,
} from '../sim/protocol';
import type { EditHost } from './edit-session';
import type { MapScene } from './map-scene';
import type { Route } from './router';
import type { SaveMode, SaveStore } from './saves';

export interface Screen {
  element: HTMLElement;
  editHost?: EditHost | undefined;
  onReady?: (ready: ReadyMessage) => void;
  onCommandResults?: (message: CommandResultsMessage) => void;
  onStats?: (message: StatsMessage) => void;
  onRunProgress?: (message: RunProgressMessage) => void;
  onBaseline?: (message: BaselineMessage) => void;
  onEvaluation?: (message: EvaluationMessage) => void;
  dispose?: () => void;
}

export interface SaveTarget {
  mode: SaveMode;
  seed: number;
  vehiclesPerHour?: number;
}

export interface GameContext {
  root: HTMLElement;
  client: SimClient;
  mapName: string;
  mapHash: string;
  saves: SaveStore;
  speed: HTMLElement;
  camera: CameraState;
  cameraQuery: boolean;
  demandQuery: number | undefined;
  attribution: string;
  challenges: () => Promise<Challenge[]>;
  navigate: (route: Route) => void;
  startSandbox: (vehiclesPerHour: number, log: EditCommand[] | undefined) => void;
  startChallenge: (challenge: Challenge, log: EditCommand[] | undefined) => void;
  showBackground: () => void;
  setSaveTarget: (target: SaveTarget | undefined) => void;
  toast: (text: string) => void;
  mapScene: () => MapScene | undefined;
  fitCity: () => void;
  panBy: (dx: number) => void;
  flyToRegion: (region: Region) => void;
}
