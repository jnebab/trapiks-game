use crate::network::LinkId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnError {
    EmptyRoute,
    TooLong,
    InactiveLink(LinkId),
    Disconnected(usize),
    Full,
    Blocked,
    NoRoute,
}
