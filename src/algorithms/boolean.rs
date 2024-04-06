mod utils;
mod event_holder;

use crate::data::Polygon;
use std::vec::Vec;

pub trait Operations<T> {
    fn intersection(subject: &Polygon<T>, clipping: &Polygon<T>) -> Vec<Polygon<T>>;
    fn union(subject: &Polygon<T>, clipping: &Polygon<T>) -> Vec<Polygon<T>>;
    fn xor(subject: &Polygon<T>, clipping: &Polygon<T>) -> Vec<Polygon<T>>;
    fn difference(subject: &Polygon<T>, clipping: &Polygon<T>) -> Vec<Polygon<T>>;
}