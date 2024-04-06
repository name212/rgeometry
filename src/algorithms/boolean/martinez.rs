use std::cmp::{Ordering, Reverse};
use std::cmp::Ordering::{Greater, Less};
use crate::data::EndPoint;
use std::collections::BTreeSet;
use std::collections::binary_heap::BinaryHeap;
use std::rc::Rc;
use crate::algorithms::boolean::martinez::PolygonType::Subject;

use crate::algorithms::boolean::utils;

//#[derive(Clone)]
// enum OpType {
//     Intersection,
//     Union,
//     Difference,
//     Xor
// }


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

