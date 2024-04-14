use std::cmp::Ordering;
use std::cmp::Ordering::{Greater, Less};
use std::rc::Rc;
use super::{Point};
use super::sweep_event::{SweepEvent, EdgeType};
use super::utils::{signed_area_orient};

// event for eventsHolder with comparator
#[derive(Ord, Eq)]
pub struct SweepEventComparedByEvents {
    parent: Rc<SweepEvent>,
}

impl SweepEventComparedByEvents {
    pub fn new(e: Rc<SweepEvent>) -> SweepEventComparedByEvents {
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

        return if !self.parent.is_subject && other.parent.is_subject {
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
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    use super::*;

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
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p1, true, Rc::new(None), false, EdgeType::Normal)));
        p2 = Rc::new(Point::new([0.0, 0.0]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p2, false, Rc::new(None), false, EdgeType::Normal)));

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
        let mut eo1 = SweepEvent::new(poe1, false, Rc::new(None), false, EdgeType::Normal);
        p1 = Rc::new(Point::new([0.0, 0.0]));
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p1, true, Rc::new(Some(eo1)), false, EdgeType::Normal)));

        let mut poe2 = Rc::new(Point::new([2.0, 3.0]));
        let mut eo2 = SweepEvent::new(poe2, false,Rc::new(None), false, EdgeType::Normal);
        p2 = Rc::new(Point::new([0.0, 0.0]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p2, true, Rc::new(Some(eo2)), false, EdgeType::Normal)));

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
        eo1 = SweepEvent::new(poe1, false, Rc::new(None), false, EdgeType::Normal);
        p1 = Rc::new(Point::new([0.0, 0.0]));
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p1, true, Rc::new(Some(eo1)),false, EdgeType::Normal)));

        poe2 = Rc::new(Point::new([2.0, 2.0]));
        eo2 = SweepEvent::new(poe2, false, Rc::new(None), false, EdgeType::Normal);
        p2 = Rc::new(Point::new([0.0, 0.0]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p2, true, Rc::new(Some(eo2)), true, EdgeType::Normal)));

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
        e1 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p1.clone(), true, Rc::new(None), true, EdgeType::Normal)));
        p2 = Rc::new(Point::new([0.0, 0.0]));
        e2 = SweepEventComparedByEvents::new(Rc::new(SweepEvent::new(p2.clone(), false, Rc::new(None), true, EdgeType::Normal)));

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