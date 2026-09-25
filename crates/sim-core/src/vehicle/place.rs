use crate::network::LinkId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    Link {
        link: LinkId,
        lane: u8,
    },
    Movement {
        node: u32,
        movement: u16,
        from_lane: u8,
        to_lane: u8,
    },
}
