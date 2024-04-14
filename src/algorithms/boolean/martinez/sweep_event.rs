use std::rc::Rc;
use super::{LineSegment, Point};
use crate::data::EndPoint;
use super::utils;

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum EdgeType {
    Normal,
    NonContributing,
    SameTransition,
    DifferentTransition
}


#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct SweepEvent {
    // is the point the left endpoint of the segment (p, other->p)?
    pub left: bool,
    // point associated with the event
    pub p: Rc<Point>,
    // Event associated to the other endpoint of the segment
    pub other: Rc<Option<SweepEvent>>,
    // Polygon to which the associated segment belongs to
    pub is_subject: bool,
    // Edge contribution type
    pub tp: EdgeType,
    // Does the segment (p, other->p) represent an inside-outside transition in the polygon for a vertical ray from (p.x, -infinite) that crosses the segment?
    pub in_out: bool,
    // Only used in "left" events. Is the segment (p, other->p) inside the other polygon?
    pub other_in_out: bool,
    // Previous event in result?
    pub prev_in_result: Rc<Option<SweepEvent>>,
    // Type of result transition (0 = not in result, +1 = out-in, -1, in-out)
    pub result_transition: i64,

    // connection step
    pub other_pos: i64,
    pub output_contour_id: i64,
    pub contour_id: i64,
    pub is_exterior_ring: bool,
}

//----------------------------------------sweep_event.js----------------------------------------------
impl SweepEvent {
    pub fn new(p: Rc<Point>, left: bool, other: Rc<Option<SweepEvent>>, is_subject: bool, tp: EdgeType) -> SweepEvent {
        Self {
            p: Rc::clone(&p),
            left,
            is_subject,
            other,
            in_out: false,
            tp,
            contour_id: 0,
            other_pos: -1,
            output_contour_id: -1,
            result_transition: 0,
            prev_in_result: Rc::new(None),
            other_in_out: false,
            is_exterior_ring: true,
        }
    }

    pub fn from_point(p: Rc<Point>) -> SweepEvent {
        return Self::new(p, false, Rc::new(None), false, EdgeType::Normal);
    }
    pub fn nothing() -> Self {
        Self::new(
            Rc::new(Point::new([0.0, 0.0])),
            false,
            Rc::new(None),
            false,
            EdgeType::Normal
        )
    }

    pub fn segment(&self) -> LineSegment {
        let pp = self.p.array.clone();
        let evpp = match self.other.as_ref() {
            None => [0.0, 0.0],
            Some(x) => x.p.clone().array.clone()
        };
        LineSegment::new(EndPoint::Inclusive(Point::new(pp)), EndPoint::Inclusive(Point::new(evpp)))
    }

    pub fn below(&self, x: &Point) -> bool {
        let evpp = match self.other.as_ref() {
            None => Rc::new(Point::new([0.0, 0.0])),
            Some(x) => x.p.clone()
        };

        if self.left {
            utils::signed_area(&self.p.clone(), evpp.as_ref(), &x.clone()) > 0.0
        } else {
            utils::signed_area(evpp.as_ref(), &self.p.clone(), &x.clone()) > 0.0
        }
    }

    pub fn above(&self, x: &Point) -> bool {
        !self.below(x)
    }
}

//
//---------------------------------sweep_event.test.js------------------------------------
#[cfg(test)]
mod sweep_event_tests{
    use std::cmp::Reverse;
    use claims::assert_ok;
    use super::*;

    #[test]
    fn test_is_below() {
        let s1 = SweepEvent::new(
            Rc::new(Point::new([0.0, 0.0])),
            true,
            Rc::new(Some(
                SweepEvent::new(
                    Rc::new(Point::new([1.0, 1.0])),
                    false,
                    Rc::new(None),
                    false,
                    EdgeType::Normal
                )
            )),
            false,
            EdgeType::Normal
        );

        let s2 = SweepEvent::new(
            Rc::new(Point::new([0.0, 1.0])),
            false,
            Rc::new(Some(
                SweepEvent::new(
                    Rc::new(Point::new([0.0, 0.0])),
                    false,
                    Rc::new(None),
                    false,
                    EdgeType::Normal
                )
            )),
            false,
            EdgeType::Normal
        );

        assert!(s1.below(&Point::new([0.0, 1.0])));
        assert!(s1.below(&Point::new([1.0, 2.0])));
        assert!(!s1.below(&Point::new([0.0, 0.0])));
        assert!(!s1.below(&Point::new([5.0, -1.0])));

        assert!(!s2.below(&Point::new([0.0, 1.0])));
        assert!(!s2.below(&Point::new([1.0, 2.0])));
        assert!(!s2.below(&Point::new([0.0, 0.0])));
        assert!(!s2.below(&Point::new([5.0, -1.0])));
    }

    #[test]
    fn test_is_above() {
        let s1 = SweepEvent::new(
            Rc::new(Point::new([0.0, 0.0])),
            true,
            Rc::new(Some(
                SweepEvent::new(
                    Rc::new(Point::new([1.0, 1.0])),
                    false,
                    Rc::new(None),
                    false,
                    EdgeType::Normal
                )
            )),
            false,
            EdgeType::Normal
        );

        let s2 = SweepEvent::new(
            Rc::new(Point::new([0.0, 1.0])),
            false,
            Rc::new(Some(
                SweepEvent::new(
                    Rc::new(Point::new([0.0, 0.0])),
                    false,
                    Rc::new(None),
                    false,
                    EdgeType::Normal
                )
            )),
            false,
            EdgeType::Normal
        );

        assert!(!s1.above(&Point::new([0.0, 1.0])));
        assert!(!s1.above(&Point::new([1.0, 2.0])));
        assert!(s1.above(&Point::new([0.0, 0.0])));
        assert!(s1.above(&Point::new([5.0, -1.0])));

        assert!(s2.above(&Point::new([0.0, 1.0])));
        assert!(s2.above(&Point::new([1.0, 2.0])));
        assert!(s2.above(&Point::new([0.0, 0.0])));
        assert!(s2.above(&Point::new([5.0, -1.0])));
    }
}