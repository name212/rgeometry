use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use super::Operations;
use crate::data::EndPoint;
use crate::data::Polygon;
use crate::TotalOrd;
use std::collections::BTreeSet;
use std::collections::binary_heap::BinaryHeap;
use std::collections::VecDeque;
use std::rc::Rc;
use geometry_predicates::orient2d;
use crate::algorithms::boolean::martinez::PolygonType::Subject;

#[derive(Clone)]
enum OpType {
    Intersection,
    Union,
    Difference,
    Xor
}
#[derive(Clone)]
enum EdgeType {
    Normal,
    NonContributing,
    SameTransition,
    DifferentTransition
}
#[derive(Clone, PartialEq)]
enum PolygonType {
    Subject,
    Clipping
}

type Point = crate::data::Point<f64>;
type LineSegment = crate::data::LineSegment<f64>;

fn signed_area (p0: &Point, p1: &Point, p2: &Point) -> f64 {
    (*p0.x_coord() - *p2.x_coord()) * (*p1.y_coord() - *p2.y_coord()) - (*p1.x_coord() - *p2.x_coord()) * (*p0.y_coord() - *p2.y_coord())
}

fn signed_area_orient (p0: &Point, p1: &Point, p2: &Point) -> f64 {
    orient2d([*p0.x_coord(), *p0.y_coord()], [*p1.x_coord(), *p1.y_coord()], [*p2.x_coord(), *p2.y_coord()])
}
#[derive(Clone)]
struct SweepEvent {
    // point associated with the event
    p: Rc<Point>,
    // is the point the left endpoint of the segment (p, other->p)?
    left: bool,
    // Polygon to which the associated segment belongs to
    pl: PolygonType,
    // Event associated to the other endpoint of the segment
    other: Option<Rc<SweepEvent>>,
    // Does the segment (p, other->p) represent an inside-outside transition in the polygon for a vertical ray from (p.x, -infinite) that crosses the segment?
    in_out: bool,
    tp: EdgeType,
    // Only used in "left" events. Is the segment (p, other->p) inside the other polygon?
    inside: bool,
    // Only used in "left" events. Position of the event (line segment) in S
    poss: BTreeSet<Rc<SweepEvent>>,
}

// event for eventsHolder with comparator
struct SweepEventComparedByEvents {
    parent: SweepEvent,
}

impl SweepEventComparedByEvents {
    fn new(e: SweepEvent) -> SweepEventComparedByEvents {
        Self {
            parent: e,
        }
    }

    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }

    // Compare two sweep events
    // Return Greater means that self is placed at the event queue after other, i.e,
    // self is processed by the algorithm after other
    fn partial_cmp(&self, other: &SweepEventComparedByEvents) -> Option<Ordering> {
        // Different x-coordinate
        if *self.parent.p.x_coord() > *other.parent.p.x_coord() {
            return Some(Greater);
        }
        if *other.parent.p.x_coord() > *self.parent.p.x_coord() {
            return Some(Less);
        }

        // Different points, but same x-coordinate
        // Event with lower y-coordinate is processed first
        if *self.parent.p.y_coord() != *other.parent.p.y_coord() {
            if *self.parent.p.y_coord() > *other.parent.p.y_coord() {
                return Some(Greater);
            } else {
                return Some(Less);
            }
        }

        // Same coordinates, but one is a left endpoint and the other is
        // a right endpoint. The right endpoint is processed first
        if self.parent.left != other.parent.left {
            if self.parent.left {
                return Some(Greater);
            } else {
                return Some(Less);
            }
        }

        // Same coordinates, both events
        // are left endpoints or right endpoints.
        // not collinear

        if self.parent.other.is_some() && other.parent.other.is_some() {
            let e1 = match self.parent.other.as_ref() {
                None => Rc::new(Point::new([0.0, 0.0])),
                Some(x) => x.p.clone()
            };
            let e2 = match other.parent.other.as_ref() {
                None => Rc::new(Point::new([0.0, 0.0])),
                Some(x) => x.p.clone()
            };
            if signed_area_orient(self.parent.p.as_ref(), e1.as_ref(), e2.as_ref()) != 0.0 {
                return if !self.parent.below(e2.as_ref()) {
                    Some(Greater)
                } else {
                    Some(Less)
                }
            }
        }

        return if self.parent.pl != Subject && other.parent.pl == Subject {
            Some(Greater)
        } else {
            Some(Less)
        }
    }
}
impl SweepEvent {
    fn new(p: Rc<Point>, left: bool, pl: PolygonType, other: Option<Rc<SweepEvent>>, tp: EdgeType) -> SweepEvent {
        Self {p, left, pl, other, in_out: left, tp, inside: false, poss: BTreeSet::new() }
    }

    fn from_point(p: Rc<Point>) -> SweepEvent {
        Self {p, left: false, pl: PolygonType::Subject, other: None, in_out: false, tp: EdgeType::Normal, inside: false, poss: BTreeSet::new()}
    }

    fn segment(&self) -> LineSegment {
        let pp = self.p.array.clone();
        let evpp = match self.other.as_ref() {
            None => [0.0, 0.0],
            Some(x) => x.p.clone().array.clone()
        };
        LineSegment::new(EndPoint::Inclusive(Point::new(pp)), EndPoint::Inclusive(Point::new(evpp)))
    }

    fn below(&self, x: &Point) -> bool {
        let evpp = match self.other.as_ref() {
            None => Rc::new(Point::new([0.0, 0.0])),
            Some(x) => x.p.clone()
        };

        if self.left {
            signed_area(&self.p.clone(), evpp.as_ref(), &x.clone()) > 0.0
        } else {
            signed_area(evpp.as_ref(), &self.p.clone(), &x.clone()) > 0.0
        }
    }

    fn above(&self, x: &Point) -> bool {
        !self.below(x)
    }

    fn clone(&self) -> Self {
        todo!()
    }
}

// struct MartinezSolver<T: TotalOrd + std::ops::Sub<Output = T> + std::ops::Mul<Output=T> + Copy + num_traits::Zero + std::cmp::PartialOrd> {
//     eq: BinaryHeap<SweepEvent<T>>,
//     /** @brief It holds the events generated during the computation of the boolean operation **/
//     event_holder: VecDeque<SweepEvent<T>>,
//     subject: Polygon<T>,
//     clipping: Polygon<T>,
//     n_int: u64,
// }
//
// impl<T: TotalOrd + std::ops::Sub<Output = T> + std::ops::Mul<Output=T> + Copy + num_traits::Zero> MartinezSolver<T> {
//     fn new(subject: Polygon<T>, clipping: Polygon<T>) -> MartinezSolver<T>{
//         MartinezSolver{
//             eq: BinaryHeap::new(),
//             event_holder: VecDeque::new(),
//             subject,
//             clipping,
//             n_int: 0,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signet_area(){
        let mut p0 = Point::new([0.0, 0.0]);
        let mut p1 = Point::new([0.0, 1.0]);
        let mut p2 = Point::new([1.0, 1.0]);
        assert_eq!(signed_area_orient(&p0, &p1, &p2), -1.0);

        p0 = Point::new([0.0, 1.0]);
        p1 = Point::new([0.0, 0.0]);
        p2 = Point::new([1.0, 0.0]);
        assert_eq!(signed_area_orient(&p0, &p1, &p2), 1.0);

        p0 = Point::new([0.0, 0.0]);
        p1 = Point::new([1.0, 1.0]);
        p2 = Point::new([2.0, 2.0]);
        assert_eq!(signed_area_orient(&p0, &p1, &p2), 0.0);

        p0 = Point::new([-1.0, 0.0]);
        p1 = Point::new([2.0, 3.0]);
        p2 = Point::new([0.0, 1.0]);
        assert_eq!(signed_area_orient(&p0, &p1, &p2), 0.0);

        p0 = Point::new([2.0, 3.0]);
        p1 = Point::new([-1.0, 0.0]);
        p2 = Point::new([0.0, 1.0]);
        assert_eq!(signed_area_orient(&p0, &p1, &p2), 0.0);
    }
    #[test]
    fn test_sweep_event_queued_comparator(){
        // queue should process lest(by x) sweep event first
    }
}