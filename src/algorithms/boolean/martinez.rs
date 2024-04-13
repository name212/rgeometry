use std::cmp::{Ordering, Reverse};
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::BTreeSet;
use std::collections::binary_heap::BinaryHeap;
use std::rc::Rc;
use geometry_predicates::orient2d;
use crate::data::EndPoint;

fn signed_area (p0: &Point, p1: &Point, p2: &Point) -> f64 {
    (*p0.x_coord() - *p2.x_coord()) * (*p1.y_coord() - *p2.y_coord()) - (*p1.x_coord() - *p2.x_coord()) * (*p0.y_coord() - *p2.y_coord())
}

fn signed_area_orient (p0: &Point, p1: &Point, p2: &Point) -> f64 {
    orient2d([*p0.x_coord(), *p0.y_coord()], [*p1.x_coord(), *p1.y_coord()], [*p2.x_coord(), *p2.y_coord()])
}

#[cfg(test)]
mod area_tests{
    use std::cmp::Reverse;
    use super::*;

    #[test]
    fn test_signet_area() {
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
}


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

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
struct SweepEvent {
    // point associated with the event
    p: Rc<Point>,
    // is the point the left endpoint of the segment (p, other->p)?
    left: bool,
    // Polygon to which the associated segment belongs to
    pl: PolygonType,
    // Event associated to the other endpoint of the segment
    other: Rc<Option<SweepEvent>>,
    // Does the segment (p, other->p) represent an inside-outside transition in the polygon for a vertical ray from (p.x, -infinite) that crosses the segment?
    in_out: bool,
    tp: EdgeType,
    // Only used in "left" events. Is the segment (p, other->p) inside the other polygon?
    inside: bool,
    // Only used in "left" events. Position of the event (line segment) in S
    poss: BTreeSet<Rc<SweepEvent>>,

    contour_id: i64
}

//----------------------------------------sweep_event.js----------------------------------------------
impl SweepEvent {
    fn new(p: Rc<Point>, left: bool, pl: PolygonType, other: Rc<Option<SweepEvent>>, tp: EdgeType) -> SweepEvent {
        Self {p: p.clone(), left, pl, other, in_out: left, tp, inside: false, poss: BTreeSet::new(), contour_id: 0 }
    }

    fn from_point(p: Rc<Point>) -> SweepEvent {
        Self {p: p.clone(), left: false, pl: PolygonType::Subject, other: Rc::new(None), in_out: false, tp: EdgeType::Normal, inside: false, poss: BTreeSet::new(), contour_id: 0}
    }
    fn nothing() -> Self {
        Self::new(
            Rc::new(Point::new([0.0, 0.0])),
            false,
            PolygonType::Clipping,
            Rc::new(None),
            EdgeType::Normal
        )
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
            PolygonType::Clipping,
            Rc::new(Some(
                SweepEvent::new(
                    Rc::new(Point::new([1.0, 1.0])),
                    false,
                    PolygonType::Clipping,
                    Rc::new(None),
                    EdgeType::Normal
                )
            )),
            EdgeType::Normal
        );

        let s2 = SweepEvent::new(
            Rc::new(Point::new([0.0, 1.0])),
            false,
            PolygonType::Clipping,
            Rc::new(Some(
                SweepEvent::new(
                    Rc::new(Point::new([0.0, 0.0])),
                    false,
                    PolygonType::Clipping,
                    Rc::new(None),
                    EdgeType::Normal
                )
            )),
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
            PolygonType::Clipping,
            Rc::new(Some(
                SweepEvent::new(
                    Rc::new(Point::new([1.0, 1.0])),
                    false,
                    PolygonType::Clipping,
                    Rc::new(None),
                    EdgeType::Normal
                )
            )),
            EdgeType::Normal
        );

        let s2 = SweepEvent::new(
            Rc::new(Point::new([0.0, 1.0])),
            false,
            PolygonType::Clipping,
            Rc::new(Some(
                SweepEvent::new(
                    Rc::new(Point::new([0.0, 0.0])),
                    false,
                    PolygonType::Clipping,
                    Rc::new(None),
                    EdgeType::Normal
                )
            )),
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
//--------------------------------------------------------------------------------------------

struct EventsHolder {
    queue: BinaryHeap<Reverse<SweepEventComparedByEvents>>
}

impl EventsHolder {
    fn new() -> EventsHolder {
        EventsHolder{queue: BinaryHeap::new()}
    }

    fn push(&mut self, e: SweepEventComparedByEvents) {
        self.queue.push(Reverse(e))
    }

    fn pop(&mut self) -> Option<SweepEventComparedByEvents> {
        return match self.queue.pop() {
            None => None,
            Some(x) => Some(x.0)
        }
    }
}

//----------------------------------compare_events.js -> compareEvents-------------------------------/
// event for eventsHolder with comparator
#[derive(Ord, Eq)]
struct SweepEventComparedByEvents {
    parent: Rc<SweepEvent>,
}

impl SweepEventComparedByEvents {
    fn new(e: Rc<SweepEvent>) -> SweepEventComparedByEvents {
        Self {
            parent: Rc::clone(&e),
        }
    }
}

impl PartialEq<Self> for SweepEventComparedByEvents {
    fn eq(&self, other: &SweepEventComparedByEvents) -> bool {
        self.cmp(other).is_eq()
    }
}
impl PartialOrd<Self> for SweepEventComparedByEvents {
    // Compare two sweep events
    // Return Greater means that self is placed at the event queue after other, i.e,
    // self is processed by the algorithm after other
    fn partial_cmp(&self, other: &SweepEventComparedByEvents) -> Option<Ordering> {
        let p1 = Rc::clone(&Rc::clone(&self.parent).p);
        let p2 = Rc::clone(&Rc::clone(&other.parent).p);

        // Different x-coordinate
        if p1.x_coord() > p2.x_coord() {
            return Some(Greater);
        }
        if p1.x_coord() < p2.x_coord() {
            return Some(Less);
        }

        // Different points, but same x-coordinate
        // Event with lower y-coordinate is processed first
        if p1.y_coord() != p2.y_coord() {
            if p1.y_coord() > p2.y_coord() {
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

        return if self.parent.pl == PolygonType::Clipping && other.parent.pl == PolygonType::Subject {
            Some(Less)
        } else {
            Some(Greater)
        }
    }
}
//
//----------------------------------compare_events.test.js-------------------------------/
#[cfg(test)]
mod sweep_event_compared_by_events_tests {
    use crate::algorithms::boolean::martinez::PolygonType::Subject;
    use super::*;

    #[test]
    fn test_sweep_event_queued_comparator(){
        // sweep event comparison x coordinates
        let mut p1 = Rc::new(Point::new([0.0, 0.0]));
        let mut e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::from_point(p1)));
        let mut p2 = Rc::new(Point::new([0.5, 0.5]));
        let mut e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::from_point(p2)));

        match e1.partial_cmp(&e2) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Less => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        match e2.partial_cmp(&e1) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Greater => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        // sweep event comparison y coordinates'
        p1 = Rc::new(Point::new([0.0, 0.0]));
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::from_point(p1)));
        p2 = Rc::new(Point::new([0.0, 0.5]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::from_point(p2)));

        match e1.partial_cmp(&e2) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Less => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        match e2.partial_cmp(&e1) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Greater => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        // sweep event comparison not left firs
        p1 = Rc::new(Point::new([0.0, 0.0]));
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p1, true, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)));
        p2 = Rc::new(Point::new([0.0, 0.0]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p2, false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)));

        match e2.partial_cmp(&e1) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Less => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        match e1.partial_cmp(&e2) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Greater => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        // sweep event comparison shared start point not collinear edges
        let mut poe1 = Rc::new(Point::new([1.0, 1.0]));
        let mut eo1 = SweepEvent::new(poe1, false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal);
        p1 = Rc::new(Point::new([0.0, 0.0]));
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p1, true, PolygonType::Clipping, Rc::new(Some(eo1)), EdgeType::Normal)));

        let mut poe2 = Rc::new(Point::new([2.0, 3.0]));
        let mut eo2 = SweepEvent::new(poe2, false, PolygonType::Clipping,Rc::new(None), EdgeType::Normal);
        p2 = Rc::new(Point::new([0.0, 0.0]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p2, true, PolygonType::Clipping, Rc::new(Some(eo2)), EdgeType::Normal)));

        match e1.partial_cmp(&e2) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Less => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        match e2.partial_cmp(&e1) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Greater => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        // sweep event comparison collinear edges
        poe1 = Rc::new(Point::new([1.0, 1.0]));
        eo1 = SweepEvent::new(poe1, false, PolygonType::Clipping,Rc::new(None), EdgeType::Normal);
        p1 = Rc::new(Point::new([0.0, 0.0]));
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p1, true, PolygonType::Clipping, Rc::new(Some(eo1)), EdgeType::Normal)));

        poe2 = Rc::new(Point::new([2.0, 2.0]));
        eo2 = SweepEvent::new(poe2, false, PolygonType::Subject, Rc::new(None), EdgeType::Normal);
        p2 = Rc::new(Point::new([0.0, 0.0]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p2, true, PolygonType::Subject, Rc::new(Some(eo2)), EdgeType::Normal)));

        match e1.partial_cmp(&e2) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Less => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        match e2.partial_cmp(&e1) {
            None => panic!("None cmp"),
            Some(x) => match x {
                Greater => println!("ok"),
                _ => panic!("Incorrect cmp")
            },
        }

        // queue should process lest(by x) sweep event first
        p1 = Rc::new(Point::new([0.0, 0.0]));
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::from_point(p1.clone())));
        p2 = Rc::new(Point::new([0.5, 0.5]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::from_point(p2.clone())));

        let mut event_holder = EventsHolder::new();
        event_holder.push(e2);
        event_holder.push(e1);

        match event_holder.pop() {
            None => panic!("Empty queue"),
            Some(x) => {
                assert_eq!(x.parent.p.clone().array[0], p1.clone().array[0]);
                assert_eq!(x.parent.p.clone().array[1], p1.clone().array[1]);
            }
        }

        match event_holder.pop() {
            None => panic!("Empty queue"),
            Some(x) => {
                assert_eq!(x.parent.p.clone().array[0], p2.clone().array[0]);
                assert_eq!(x.parent.p.clone().array[1], p2.clone().array[1]);
            }
        }

        // queue should process lest(by y) sweep event first
        p1 = Rc::new(Point::new([0.0, 0.0]));
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::from_point(p1.clone())));
        p2 = Rc::new(Point::new([0.0, 0.5]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::from_point(p2.clone())));

        event_holder = EventsHolder::new();
        event_holder.push(e2);
        event_holder.push(e1);

        match event_holder.pop() {
            None => panic!("Empty queue"),
            Some(x) => {
                assert_eq!(x.parent.p.clone().array[0], p1.clone().array[0]);
                assert_eq!(x.parent.p.clone().array[1], p1.clone().array[1]);
            }
        }

        match event_holder.pop() {
            None => panic!("Empty queue"),
            Some(x) => {
                assert_eq!(x.parent.p.clone().array[0], p2.clone().array[0]);
                assert_eq!(x.parent.p.clone().array[1], p2.clone().array[1]);
            }
        }

        // 'queue should pop least(by left prop) sweep event first
        p1 = Rc::new(Point::new([0.0, 0.0]));
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p1.clone(), true, Subject, Rc::new(None), EdgeType::Normal)));
        p2 = Rc::new(Point::new([0.0, 0.0]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p2.clone(), false, Subject, Rc::new(None), EdgeType::Normal)));

        event_holder = EventsHolder::new();
        event_holder.push(e1);
        event_holder.push(e2);

        match event_holder.pop() {
            None => panic!("Empty queue"),
            Some(x) => {
                assert_eq!(x.parent.p.clone().array[0], p2.clone().array[0]);
                assert_eq!(x.parent.p.clone().array[1], p2.clone().array[1]);
            }
        }

        match event_holder.pop() {
            None => panic!("Empty queue"),
            Some(x) => {
                assert_eq!(x.parent.p.clone().array[0], p1.clone().array[0]);
                assert_eq!(x.parent.p.clone().array[1], p1.clone().array[1]);
            }
        }
    }
}
//----------------------------------compare_segments.js -> compareSegments-------------------------------/
// event for eventsHolder with comparator
#[derive(Ord, Eq)]
struct SweepEventComparedBySegments {
    parent: Rc<SweepEvent>,
}

impl SweepEventComparedBySegments {
    fn new(e: Rc<SweepEvent>) -> SweepEventComparedBySegments {
        Self {
            parent: Rc::clone(&e),
        }
    }
    fn nothing() -> SweepEventComparedBySegments {
        Self::new(Rc::new(SweepEvent::nothing()))
    }
}

impl PartialEq<Self> for SweepEventComparedBySegments {
    fn eq(&self, other: &SweepEventComparedBySegments) -> bool {
        self.cmp(other).is_eq()
    }
}
impl PartialOrd<Self> for SweepEventComparedBySegments {
    // Compare two sweep events
    // Return Greater means that self is placed at the event queue after other, i.e,
    // self is processed by the algorithm after other
    fn partial_cmp(&self, other: &SweepEventComparedBySegments) -> Option<Ordering> {
        if std::ptr::eq(self, other) {
            return Some(Equal)
        }

        let self_other = match self.parent.other.as_ref() {
            None => panic!("First has not other event"),
            Some(x) => x
        };

        let other_other = match other.parent.other.as_ref() {
            None => panic!("Second has not other event"),
            Some(x) => x
        };

        // Segments are not collinear
        if signed_area(&self.parent.p, &self_other.p, &other.parent.p) != 0.0
            || signed_area(&self.parent.p, &self_other.p, &other_other.p) != 0.0
        {
            // If they share their left endpoint use the right endpoint to sort
            if self.parent.p.eq(&other.parent.p) {
                if self.parent.below(&other_other.p) {
                    return Some(Less)
                }
                return Some(Greater)
            }

            // Different left endpoint: use the left endpoint to sort
            if self.parent.p.x_coord() == other.parent.p.x_coord(){
                if self.parent.p.y_coord() < other.parent.p.y_coord() {
                    return Some(Less)
                }

                return Some(Greater)
            }

            // has the line segment associated to e1 been inserted
            // into S after the line segment associated to e2 ?
            let e1 = SweepEventComparedByEvents::new(Rc::clone(&self.parent));
            let e2 = SweepEventComparedByEvents::new(Rc::clone(&other.parent));
            if e1.partial_cmp(&e2) == Some(Greater){
                if other.parent.above(&self.parent.p) {
                    return Some(Less)
                }
                return Some(Greater)
            }

            // The line segment associated to e2 has been inserted
            // into S after the line segment associated to e1
            if self.parent.below(&other.parent.p) {
                return Some(Less)
            }

            return Some(Greater)
        }

        if self.parent.pl == other.parent.pl { // same polygon
            if self.parent.p.eq(&other.parent.p){
                if self_other.p.eq(&other_other.p) {
                   return Some(Equal)
                } else {
                    if self.parent.contour_id > other.parent.contour_id {
                        return Some(Greater)
                    } else {
                        return Some(Less)
                    }
                }
            } else {
                if SweepEventComparedByEvents::new(Rc::clone(&self.parent)).cmp(&SweepEventComparedByEvents::new(Rc::clone(&other.parent))) == Greater {
                    return Some(Greater);
                } else {
                    Some(Less);
                }
            }
        } else { // Segments are collinear, but belong to separate polygons
            return if self.parent.pl == PolygonType::Subject {
                return Some(Less);
            } else {
                return Some(Greater);
            }
        }

        if SweepEventComparedByEvents::new(Rc::clone(&self.parent)).cmp(&SweepEventComparedByEvents::new(Rc::clone(&other.parent))) == Greater {
            return Some(Greater)
        } else {
            return Some(Less)
        }
    }
}

//-----------------------------------compare_segments.test.js-----------------------------
#[cfg(test)]
mod sweep_event_compared_by_segments_tests{
    use std::cmp::Reverse;
    use super::*;
    use std::collections::BinaryHeap;

    #[test]
    fn test_not_collinear_1() {
        // shared left point - right point firs

        let mut heap = BinaryHeap::new();
        let pt = Rc::new(Point::new([0.0, 0.0]));
        let se1 = SweepEventComparedBySegments::new(
           Rc::new(SweepEvent::new(Rc::clone(&pt), true, PolygonType::Clipping, Rc::new(Some(
               SweepEvent::new(Rc::new(Point::new([1.0, 1.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
           )), EdgeType::Normal)
        ));

        let se2 = SweepEventComparedBySegments::new(
            Rc::new(SweepEvent::new(Rc::clone(&pt), true, PolygonType::Clipping, Rc::new(Some(
                SweepEvent::new(Rc::new(Point::new([2.0, 3.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
            )), EdgeType::Normal)
        ));

        heap.push(se1);
        heap.push(se2);

        let mut has = false;
        if let Some(max_node) = heap.pop() {
            if let Some(o) = max_node.parent.other.as_ref() {
                has = true;
                assert!(o.p == Rc::new(Point::new([2.0, 3.0])));
            }
        }

        assert!(has);
        has = false;

        if let Some(min_node) = heap.pop() {
            if let Some(o) = min_node.parent.other.as_ref() {
                has = true;
                assert!(o.p == Rc::new(Point::new([1.0, 1.0])));
            }
        }

        assert!(has);
    }

    #[test]
    fn test_not_collinear_2() {
        // different left point - right point y coord to sort

        let mut heap = BinaryHeap::new();

        let se1 = SweepEventComparedBySegments::new(
            Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 1.0])), true, PolygonType::Clipping, Rc::new(Some(
                SweepEvent::new(Rc::new(Point::new([1.0, 1.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
            )), EdgeType::Normal)
            ));

        let se2 = SweepEventComparedBySegments::new(
            Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 2.0])), true, PolygonType::Clipping, Rc::new(Some(
                SweepEvent::new(Rc::new(Point::new([2.0, 3.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
            )), EdgeType::Normal)
            ));

        heap.push(se1);
        heap.push(se2);

        let mut has = false;
        if let Some(max_node) = heap.pop() {
            if let Some(o) = max_node.parent.other.as_ref() {
                has = true;
                assert!(o.p == Rc::new(Point::new([2.0, 3.0])));
            }
        }

        assert!(has);
        has = false;

        if let Some(min_node) = heap.pop() {
            if let Some(o) = min_node.parent.other.as_ref() {
                has = true;
                assert!(o.p == Rc::new(Point::new([1.0, 1.0])));
            }
        }

        assert!(has);
    }

    #[test]
    fn test_not_collinear_3() {
        // events order in sweep line

        let e1 = Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 1.0])), true, PolygonType::Clipping, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([2.0, 1.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
        )), EdgeType::Normal));

        let e2 = Rc::new(SweepEvent::new(Rc::new(Point::new([-1.0, 0.0])), true, PolygonType::Clipping, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([2.0, 3.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
        )), EdgeType::Normal));

        let e3 = Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 1.0])), true, PolygonType::Clipping, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([3.0, 4.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
        )), EdgeType::Normal));

        let e4 = Rc::new(SweepEvent::new(Rc::new(Point::new([-1.0, 0.0])), true, PolygonType::Clipping, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([3.0, 1.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
        )), EdgeType::Normal));

        let se1 = SweepEventComparedBySegments::new(Rc::clone(&e1));

        let se2 = SweepEventComparedBySegments::new(Rc::clone(&e2));

        assert_eq!(SweepEventComparedByEvents::new(Rc::clone(&e1)).partial_cmp(&SweepEventComparedByEvents::new(Rc::clone(&e2))), Some(Greater));
        assert!(!se2.parent.below(&se1.parent.p));
        assert!(se2.parent.above(&se1.parent.p));

        assert_eq!(se1.partial_cmp(&se2), Some(Less));
        assert_eq!(se2.partial_cmp(&se1), Some(Greater));

        assert_eq!(SweepEventComparedByEvents::new(Rc::clone(&e3)).partial_cmp(&SweepEventComparedByEvents::new(Rc::clone(&e4))), Some(Greater));
        assert!(!e4.above(&e3.p))
    }

    #[test]
    fn test_not_collinear_4() {
        // first point is below

        let e1 = Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 1.0])), true, PolygonType::Clipping, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([2.0, 1.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
        )), EdgeType::Normal));

        let e2 = Rc::new(SweepEvent::new(Rc::new(Point::new([-1.0, 0.0])), true, PolygonType::Clipping, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([2.0, 3.0])), false, PolygonType::Clipping, Rc::new(None), EdgeType::Normal)
        )), EdgeType::Normal));

        let se2 = SweepEventComparedBySegments::new(Rc::clone(&e1));

        let se1 = SweepEventComparedBySegments::new(Rc::clone(&e2));

        assert!(!se1.parent.below(&se2.parent.p));
        assert_eq!(se1.partial_cmp(&se2), Some(Greater))
    }

}
