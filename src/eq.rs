use crate::MiniVec;

use core::cmp::Eq;

impl<T> Eq for MiniVec<T> where T: Eq {}
