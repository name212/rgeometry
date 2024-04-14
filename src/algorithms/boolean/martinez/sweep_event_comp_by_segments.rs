use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::rc::Rc;
use super::{Point};
use super::sweep_event::{SweepEvent, EdgeType};
use super::sweep_event_comp_by_event::{SweepEventComparedByEvents};
use super::utils::{signed_area};

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

        if self.parent.is_subject == other.parent.is_subject { // same polygon
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
            return if self.parent.is_subject {
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
            Rc::new(SweepEvent::new(Rc::clone(&pt), true, Rc::new(Some(
                SweepEvent::new(Rc::new(Point::new([1.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
            )), false, EdgeType::Normal)
            ));

        let se2 = SweepEventComparedBySegments::new(
            Rc::new(SweepEvent::new(Rc::clone(&pt), true, Rc::new(Some(
                SweepEvent::new(Rc::new(Point::new([2.0, 3.0])), false, Rc::new(None), false, EdgeType::Normal)
            )), false, EdgeType::Normal)
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
            Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 1.0])), true, Rc::new(Some(
                SweepEvent::new(Rc::new(Point::new([1.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
            )), false, EdgeType::Normal)
            ));

        let se2 = SweepEventComparedBySegments::new(
            Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 2.0])), true, Rc::new(Some(
                SweepEvent::new(Rc::new(Point::new([2.0, 3.0])), false, Rc::new(None), false, EdgeType::Normal)
            )), false, EdgeType::Normal)
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

        let e1 = Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 1.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([2.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal));

        let e2 = Rc::new(SweepEvent::new(Rc::new(Point::new([-1.0, 0.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([2.0, 3.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal));

        let e3 = Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 1.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([3.0, 4.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal));

        let e4 = Rc::new(SweepEvent::new(Rc::new(Point::new([-1.0, 0.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([3.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal));

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

        let e1 = Rc::new(SweepEvent::new(Rc::new(Point::new([0.0, 1.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([2.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal));

        let e2 = Rc::new(SweepEvent::new(Rc::new(Point::new([-1.0, 0.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([2.0, 3.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal));

        let se2 = SweepEventComparedBySegments::new(Rc::clone(&e1));

        let se1 = SweepEventComparedBySegments::new(Rc::clone(&e2));

        assert!(!se1.parent.below(&se2.parent.p));
        assert_eq!(se1.partial_cmp(&se2), Some(Greater))
    }

    #[test]
    fn test_collinear_segments() {
        // collinear segment
        let e1 = Rc::new(SweepEvent::new(Rc::new(Point::new([1.0, 1.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([5.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), true, EdgeType::Normal));

        let e2 = Rc::new(SweepEvent::new(Rc::new(Point::new([2.0, 1.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([3.0, 3.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal));

        let se1 = SweepEventComparedBySegments::new(Rc::clone(&e1));

        let se2 = SweepEventComparedBySegments::new(Rc::clone(&e2));

        assert_ne!(se1.parent.is_subject, se2.parent.is_subject);
        assert_eq!(se1.partial_cmp(&se2), Some(Greater))
    }

    #[test]
    fn test_collinear_shared_left_point() {
        // collinear shared left point
        let pt = Rc::new(Point::new([0.0, 1.0]));

        let mut e1 = SweepEvent::new(Rc::clone(&pt), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([5.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal);

        let mut e2 = SweepEvent::new(Rc::clone(&pt), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([3.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal);

        e1.contour_id = 1;
        e2.contour_id = 2;

        let mut se1 = SweepEventComparedBySegments::new(Rc::clone(&Rc::new(e1)));

        let mut se2 = SweepEventComparedBySegments::new(Rc::clone(&Rc::new(e2)));


        assert_eq!(se1.parent.is_subject, se2.parent.is_subject);
        assert_eq!(se1.parent.p, se2.parent.p);

        assert_eq!(se1.partial_cmp(&se2), Some(Less));

        let mut e3 = SweepEvent::new(Rc::clone(&pt), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([5.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal);

        let mut e4 = SweepEvent::new(Rc::clone(&pt), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([3.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), false, EdgeType::Normal);

        e3.contour_id = 2;
        e4.contour_id = 1;

        let se3 = SweepEventComparedBySegments::new(Rc::clone(&Rc::new(e3)));
        let se4 = SweepEventComparedBySegments::new(Rc::clone(&Rc::new(e4)));

        assert_eq!(se3.partial_cmp(&se4), Some(Greater));
    }

    #[test]
    fn test_collinear_same_polygon_different_left_points() {
        // collinear same polygon different left points
        let e1 = Rc::new(SweepEvent::new(Rc::new(Point::new([1.0, 1.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([5.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), true, EdgeType::Normal));

        let e2 = Rc::new(SweepEvent::new(Rc::new(Point::new([2.0, 1.0])), true, Rc::new(Some(
            SweepEvent::new(Rc::new(Point::new([3.0, 1.0])), false, Rc::new(None), false, EdgeType::Normal)
        )), true, EdgeType::Normal));

        let se1 = SweepEventComparedBySegments::new(Rc::clone(&e1));

        let se2 = SweepEventComparedBySegments::new(Rc::clone(&e2));

        assert_eq!(se1.parent.is_subject, se2.parent.is_subject);
        assert_ne!(se1.parent.p, se2.parent.p);

        assert_eq!(se1.partial_cmp(&se2), Some(Less));
        assert_eq!(se2.partial_cmp(&se1), Some(Greater));
    }
}