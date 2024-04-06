#[derive(Clone, PartialEq, Eq, Ord, PartialOrd)]
enum EdgeType {
    Normal,
    NonContributing,
    SameTransition,
    DifferentTransition
}
#[derive(Clone, PartialEq, Eq, Ord, PartialOrd)]
enum PolygonType {
    Subject,
    Clipping
}

type Point = crate::data::Point<f64>;
type LineSegment = crate::data::LineSegment<f64>;
