use crate::edit::SplitRecord;
use crate::network::{Direction, LinkId, link_id};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitLinks {
    pub fwd: LinkId,
    pub bwd: LinkId,
    pub fwd2: LinkId,
    pub bwd2: LinkId,
    pub at: f64,
    pub len: f64,
    pub old_to: u32,
}

impl SplitLinks {
    pub fn new(split: &SplitRecord) -> SplitLinks {
        SplitLinks {
            fwd: link_id(split.road, Direction::Forward),
            bwd: link_id(split.road, Direction::Backward),
            fwd2: link_id(split.r2, Direction::Forward),
            bwd2: link_id(split.r2, Direction::Backward),
            at: split.cut,
            len: split.old_length,
            old_to: split.old_to,
        }
    }

    pub fn split_of(self, link: LinkId) -> LinkId {
        match link {
            l if l == self.fwd => self.fwd2,
            l if l == self.bwd => self.bwd2,
            l => l,
        }
    }

    pub fn whole_of(self, link: LinkId) -> LinkId {
        match link {
            l if l == self.fwd2 => self.fwd,
            l if l == self.bwd2 => self.bwd,
            l => l,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpliceRole {
    FollowsForward,
    OnSecondBackward,
    Other,
}

pub fn splice_route(
    route: &[LinkId],
    cursor: usize,
    links: &SplitLinks,
    role: SpliceRole,
) -> (Vec<LinkId>, usize) {
    let mut out = Vec::with_capacity(route.len() + 2);
    let mut new_cursor = cursor;
    for (j, &link) in route.iter().enumerate() {
        let active = j >= cursor;
        if active && link == links.bwd {
            if j == cursor {
                new_cursor = out.len() + usize::from(role != SpliceRole::OnSecondBackward);
            }
            out.extend([links.bwd2, link]);
            continue;
        }
        if j == cursor {
            let follows = role == SpliceRole::FollowsForward && link == links.fwd;
            new_cursor = out.len() + usize::from(follows);
        }
        out.push(link);
        if active && link == links.fwd {
            out.push(links.fwd2);
        }
    }
    (out, new_cursor)
}

pub fn merge_route(route: &[LinkId], cursor: usize, links: &SplitLinks) -> (Vec<LinkId>, usize) {
    let mut out: Vec<LinkId> = Vec::with_capacity(route.len());
    let mut new_cursor = 0;
    for (j, &link) in route.iter().enumerate() {
        let whole = links.whole_of(link);
        let touches_split = whole != link || j > 0 && links.whole_of(route[j - 1]) != route[j - 1];
        if !(touches_split && out.last() == Some(&whole)) {
            out.push(whole);
        }
        if j == cursor {
            new_cursor = out.len() - 1;
        }
    }
    (out, new_cursor)
}

pub fn has_split_link(route: &[LinkId], links: &SplitLinks) -> bool {
    route.iter().any(|&link| links.whole_of(link) != link)
}
