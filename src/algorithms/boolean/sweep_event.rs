#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
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


impl SweepEvent {
    fn new(p: Rc<Point>, left: bool, pl: PolygonType, other: Option<Rc<SweepEvent>>, tp: EdgeType) -> Rc<SweepEvent> {
        Rc::new(Self {p: p.clone(), left, pl, other, in_out: left, tp, inside: false, poss: BTreeSet::new() })
    }

    fn from_point(p: Rc<Point>) -> Rc<SweepEvent> {
        Rc::new(Self {p: p.clone(), left: false, pl: PolygonType::Subject, other: None, in_out: false, tp: EdgeType::Normal, inside: false, poss: BTreeSet::new()})
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