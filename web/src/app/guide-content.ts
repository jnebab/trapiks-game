export interface GuideTable {
  head: readonly [string, string];
  rows: readonly (readonly [string, string])[];
}

export interface GuideSection {
  heading: string;
  paragraphs: readonly string[];
  table?: GuideTable;
}

export const GUIDE_SECTIONS: readonly GuideSection[] = [
  {
    heading: 'The idea',
    paragraphs: [
      "Metro Manila's real road network is already built. You fix traffic by changing roads and junctions, not by building a new city.",
      'The simulation measures delay: the time drivers lose to congestion. Your job is to bring it down.',
    ],
  },
  {
    heading: 'Moving around',
    paragraphs: [
      'Drag to pan and use the mouse wheel to zoom. Hover a road to see where it leads (green) and what feeds it (orange).',
      'L1 and L2 badges mark elevated roads; ↑ and ↓ mark ramps.',
    ],
    table: {
      head: ['Control', 'What it does'],
      rows: [
        ['⏸ 1× 2× 4× 8× Max', 'Simulation speed; Space pauses'],
        ['Traffic or T (Sandbox)', 'Colours congestion; red means jammed'],
        [
          'Levels (bottom-right) or L',
          'All / See-through / Ground / Elevated for stacked flyovers and skyways',
        ],
      ],
    },
  },
  {
    heading: 'Modes',
    paragraphs: [
      'Challenges: real chokepoints with a peso budget. The goal is to cut delay by 25 %.',
      'Sandbox: the whole city with no goal. The Demand slider adds traffic.',
    ],
  },
  {
    heading: 'Playing a challenge',
    paragraphs: [
      'The baseline delay is measured first and shown bottom-left. Then make your fixes and press Evaluate, which runs 10 simulated minutes.',
      'You pass when delay drops by at least 25 %, no more than 5 % fewer trips get through, and you stay within budget.',
    ],
    table: {
      head: ['Stars', 'Delay cut'],
      rows: [
        ['★☆☆', 'Pass (25 %)'],
        ['★★☆', '35 %'],
        ['★★★', '45 %'],
      ],
    },
  },
  {
    heading: 'Select tool (S): roads',
    paragraphs: ['Click a road to change it.'],
    table: {
      head: ['Change', 'Cost'],
      rows: [
        ['Add a lane', '₱40 per 100 m'],
        ['Remove a lane', '₱10 per 100 m'],
        ['One-way ↔ two-way', '+₱50'],
        ['Speed limit up or down', '₱10'],
        ['Delete road (or Delete key)', '₱50 per 100 m, min ₱100'],
      ],
    },
  },
  {
    heading: 'Select tool (S): junctions',
    paragraphs: [
      'Click a junction to change how it works. Set signal timing, then press Apply timing.',
    ],
    table: {
      head: ['Change', 'Cost'],
      rows: [
        ['Traffic signal', '₱500'],
        ['All-way stop', '₱80'],
        ['Stop / priority', '₱50'],
        ['Yield', '₱30'],
        ['Signal timing', '₱20'],
        ['Ban a turn', '₱20'],
        ['Build flyover', '₱2,000 + length'],
        ['Small or large roundabout', '₱1,200 + ring length'],
      ],
    },
  },
  {
    heading: 'Road tool (R)',
    paragraphs: [
      'Click a start (a junction or any point on a road), then click an end. Options: curve and elevated.',
      'The cost is shown before you build: ₱200 + length × lanes, more if elevated. Esc cancels.',
    ],
  },
  {
    heading: 'Undo and saving',
    paragraphs: [
      'Ctrl+Z or Undo reverts your last change and refunds it. Reset clears a challenge.',
      'Progress saves automatically; Continue on the title screen resumes.',
    ],
  },
  {
    heading: 'Tips',
    paragraphs: [
      'In Sandbox, turn on Traffic to find red roads, and look at the junction where the jam starts.',
      'Try cheap fixes first, such as a ₱20 retime or a left-turn ban, before a ₱3,500 flyover.',
    ],
  },
];
