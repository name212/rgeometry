use geometry_predicates::orient2d;
use crate::algorithms::boolean::martinez::Point;

pub fn signed_area (p0: &Point, p1: &Point, p2: &Point) -> f64 {
    (*p0.x_coord() - *p2.x_coord()) * (*p1.y_coord() - *p2.y_coord()) - (*p1.x_coord() - *p2.x_coord()) * (*p0.y_coord() - *p2.y_coord())
}

pub fn signed_area_orient (p0: &Point, p1: &Point, p2: &Point) -> f64 {
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

