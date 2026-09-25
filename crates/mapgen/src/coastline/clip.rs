use super::Point;
use super::boundary::{Edge, LonLatBox};

#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub points: Vec<Point>,
    pub entry_edge: Edge,
    pub exit_edge: Edge,
}

struct Span {
    t0: f64,
    t1: f64,
    entry: Option<Edge>,
    exit: Option<Edge>,
}

struct Open {
    points: Vec<Point>,
    entry: Option<Edge>,
}

pub fn clip_all(chains: &[Vec<Point>], bbox: &LonLatBox) -> (Vec<Piece>, u32) {
    let mut pieces = Vec::new();
    let mut dropped = 0;
    for chain in chains {
        for (points, entry, exit) in clip_chain(chain, bbox) {
            match (entry, exit) {
                (Some(entry_edge), Some(exit_edge)) => pieces.push(Piece {
                    points,
                    entry_edge,
                    exit_edge,
                }),
                _ => dropped += 1,
            }
        }
    }
    (pieces, dropped)
}

type RawPiece = (Vec<Point>, Option<Edge>, Option<Edge>);

fn clip_chain(chain: &[Point], bbox: &LonLatBox) -> Vec<RawPiece> {
    let mut pieces = Vec::new();
    let mut current: Option<Open> = None;
    for segment in chain.windows(2) {
        let (a, b) = (segment[0], segment[1]);
        let Some(span) = liang_barsky(a, b, bbox) else {
            pieces.extend(current.take().map(|o| close_at_end(o, bbox)));
            continue;
        };
        let start = point_at(a, b, span.t0, span.entry, bbox);
        let open = current.get_or_insert_with(|| Open {
            points: vec![start],
            entry: span.entry.or_else(|| bbox.edge_of(start)),
        });
        let end = point_at(a, b, span.t1, span.exit, bbox);
        open.points.push(end);
        if span.exit.is_some() {
            pieces.extend(current.take().map(|o| finish(o, span.exit)));
        }
    }
    pieces.extend(current.map(|o| close_at_end(o, bbox)));
    pieces
}

fn close_at_end(open: Open, bbox: &LonLatBox) -> RawPiece {
    let exit = open.points.last().and_then(|p| bbox.edge_of(*p));
    finish(open, exit)
}

fn finish(open: Open, exit: Option<Edge>) -> RawPiece {
    (open.points, open.entry, exit)
}

fn point_at(a: Point, b: Point, t: f64, edge: Option<Edge>, bbox: &LonLatBox) -> Point {
    let point = (a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1));
    match edge {
        Some(edge) => bbox.snap(edge, point),
        None => point,
    }
}

fn liang_barsky(a: Point, b: Point, bbox: &LonLatBox) -> Option<Span> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let constraints = [
        (-dx, a.0 - bbox.min_lon, Edge::Left),
        (dx, bbox.max_lon - a.0, Edge::Right),
        (-dy, a.1 - bbox.min_lat, Edge::Bottom),
        (dy, bbox.max_lat - a.1, Edge::Top),
    ];
    let mut span = Span {
        t0: 0.0,
        t1: 1.0,
        entry: None,
        exit: None,
    };
    for (p, q, edge) in constraints {
        apply(&mut span, p, q, edge)?;
    }
    (span.t0 < span.t1).then_some(span)
}

fn apply(span: &mut Span, p: f64, q: f64, edge: Edge) -> Option<()> {
    if p == 0.0 {
        return (q >= 0.0).then_some(());
    }
    let r = q / p;
    if p < 0.0 && r > span.t0 {
        span.t0 = r;
        span.entry = Some(edge);
    }
    if p > 0.0 && r < span.t1 {
        span.t1 = r;
        span.exit = Some(edge);
    }
    (span.t0 <= span.t1).then_some(())
}
