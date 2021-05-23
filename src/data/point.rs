use array_init::array_init;
use num_rational::BigRational;
use num_traits::identities::One;
use num_traits::identities::Zero;
use num_traits::FromPrimitive;
use num_traits::Num;
use num_traits::NumOps;
use num_traits::NumRef;
use num_traits::RefNum;
use num_traits::ToPrimitive;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cmp::Ordering;
// use std::ops::Add;
use std::ops::Index;
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Neg;
use std::ops::Sub;

use super::{Vector, VectorView};
use crate::array::*;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Point<T, const N: usize> {
  pub array: [T; N],
}

// Random sampling.
impl<T, const N: usize> Distribution<Point<T, N>> for Standard
where
  Standard: Distribution<T>,
{
  // FIXME: Unify with code for Vector.
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point<T, N> {
    Point {
      array: array_init(|_| rng.gen()),
    }
  }
}

// Methods on N-dimensional points.
impl<T: Clone, const N: usize> Point<T, N> {
  pub fn new(array: [T; N]) -> Point<T, N> {
    Point { array }
  }

  pub fn as_vec(&self) -> &Vector<T, N> {
    self.into()
  }

  pub fn cmp_distance_to(&self, p: &Point<T, N>, q: &Point<T, N>) -> Ordering
  where
    T: Clone + Zero + Ord + NumOps,
    // for<'a> &'a T: Mul<&'a T, Output = T> + Sub<&'a T, Output = T>,
  {
    self
      .squared_euclidean_distance(p)
      .cmp(&self.squared_euclidean_distance(q))
  }

  pub fn squared_euclidean_distance(&self, rhs: &Point<T, N>) -> T
  where
    T: Clone + Zero + NumOps,
    // for<'a> &'a T: Mul<&'a T, Output = T> + Sub<&'a T, Output = T>,
  {
    self
      .array
      .iter()
      .zip(rhs.array.iter())
      .fold(T::zero(), |sum, (a, b)| {
        let diff: T = a.clone() - b.clone();
        sum + diff.clone() * diff
      })
  }

  // Similar to num_traits::identities::Zero but doesn't require an Add impl.
  pub fn zero() -> Self
  where
    T: Zero,
  {
    Point {
      array: array_init(|_| Zero::zero()),
    }
  }

  pub fn cast<U, F>(self, f: F) -> Point<U, N>
  where
    F: Fn(T) -> U,
  {
    Point {
      array: array_init(|i| f(self.array[i].clone())),
    }
  }
}

impl<T, const N: usize> Index<usize> for Point<T, N> {
  type Output = T;
  fn index(&self, key: usize) -> &T {
    self.array.index(key)
  }
}

impl<'a, const N: usize> From<&'a Point<BigRational, N>> for Point<f64, N> {
  fn from(point: &Point<BigRational, N>) -> Point<f64, N> {
    Point {
      array: array_init(|i| point.array[i].to_f64().unwrap()),
    }
  }
}

impl<'a, const N: usize> From<&'a Point<f64, N>> for Point<BigRational, N> {
  fn from(point: &Point<f64, N>) -> Point<BigRational, N> {
    Point {
      array: array_init(|i| BigRational::from_f64(point.array[i]).unwrap()),
    }
  }
}

impl<T, const N: usize> From<Vector<T, N>> for Point<T, N> {
  fn from(vector: Vector<T, N>) -> Point<T, N> {
    Point { array: vector.0 }
  }
}

// impl<T, const N: usize> AsRef<Vector<T, N>> for Point<T, N> {
//   fn as_ref(&self) -> &Vector<T, N> {
//     self.into()
//   }
// }

// pub fn orientation<T>(p: &Point<T, 2>, q: &Point<T, 2>, r: &Point<T, 2>) -> Orientation
// where
//   T: Sub<T, Output = T> + Clone + Mul<T, Output = T> + Ord,
//   // for<'a> &'a T: Sub<Output = T>,
// {
//   raw_arr_turn(&p.array, &q.array, &r.array)
// }

// Methods on two-dimensional points.
impl<T> Point<T, 2> {
  pub fn orientation(&self, q: &Point<T, 2>, r: &Point<T, 2>) -> Orientation
  where
    T: Clone + NumOps + Ord,
    // for<'a> &'a T: Sub<Output = T>,
  {
    raw_arr_turn(&self.array, &q.array, &r.array)
  }

  /// Docs?
  pub fn ccw_cmp_around(&self, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering
  where
    T: Clone + Ord + NumOps + Zero + One + Neg<Output = T>,
    // for<'a> &'a T: Mul<&'a T, Output = T>,
  {
    self.ccw_cmp_around_with(&Vector([T::one(), T::zero()]), p, q)
  }

  pub fn ccw_cmp_around_with(&self, z: &Vector<T, 2>, p: &Point<T, 2>, q: &Point<T, 2>) -> Ordering
  where
    T: Clone + Ord + NumOps + Neg<Output = T>,
    // for<'a> &'a T: Mul<Output = T>,
  {
    ccw_cmp_around_origin_with(&z.0, &(p - self).0, &(q - self).0)
  }
}

// FIXME: Use a macro
impl<T> Point<T, 1> {
  pub fn x_coord(&self) -> &T {
    &self.array[0]
  }
}
impl<T> Point<T, 2> {
  pub fn x_coord(&self) -> &T {
    &self.array[0]
  }
  pub fn y_coord(&self) -> &T {
    &self.array[1]
  }
}
impl<T> Point<T, 3> {
  pub fn x_coord(&self) -> &T {
    &self.array[0]
  }
  pub fn y_coord(&self) -> &T {
    &self.array[1]
  }
  pub fn z_coord(&self) -> &T {
    &self.array[2]
  }
}

mod add;
mod sub;

// // Sigh, should relax Copy to Clone.
// impl<T, const N: usize> Zero for Point<T, N>
// where
//   T: Zero + Copy + Add + Add<Output = T>,
// {
//   fn zero() -> Point<T, N> {
//     Point([Zero::zero(); N])
//   }
//   fn is_zero(&self) -> bool {
//     self.0.iter().all(Zero::is_zero)
//   }
// }