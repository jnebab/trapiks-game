import type { Point } from '../../render/polyline';

export interface ArmRoads {
  from: (road: number) => number;
  name: (road: number) => number;
  classCode: (road: number) => number;
  pointsOf: (road: number) => Point[];
}

export interface ArmStores {
  roads: ArmRoads;
  names: readonly string[];
  classNames: readonly string[];
}

export function humanize(name: string): string {
  const words = name.replace(/([a-z])([A-Z])/g, '$1 $2').toLowerCase();
  return words.charAt(0).toUpperCase() + words.slice(1);
}

export function roadTitle(road: number, stores: ArmStores): string {
  const name = stores.names[stores.roads.name(road)] ?? '';
  if (name !== '') {
    return name;
  }
  const className = stores.classNames[stores.roads.classCode(road)];
  return className === undefined ? `Road #${String(road)}` : humanize(className);
}

function departingDirection(road: number, node: number, roads: ArmRoads): [number, number] {
  const points = roads.pointsOf(road);
  const departsAtStart = roads.from(road) === node;
  const start = departsAtStart ? points[0] : points[points.length - 1];
  const next = departsAtStart ? points[1] : points[points.length - 2];
  if (start === undefined || next === undefined) {
    return [0, 0];
  }
  return [next.x - start.x, next.y - start.y];
}

export function compassLetter(dx: number, dy: number): string {
  if (Math.abs(dx) > Math.abs(dy)) {
    return dx > 0 ? 'E' : 'W';
  }
  return dy < 0 ? 'N' : 'S';
}

export function armLabel(road: number, node: number, stores: ArmStores): string {
  const [dx, dy] = departingDirection(road, node, stores.roads);
  return `${roadTitle(road, stores)} ${compassLetter(dx, dy)}`;
}

export function armLabels(
  roads: readonly number[],
  node: number,
  stores: ArmStores,
): Map<number, string> {
  return new Map(roads.map((road) => [road, armLabel(road, node, stores)]));
}
