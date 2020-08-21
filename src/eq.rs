use crate::MiniVec;

use std::cmp::Eq;

impl<T> Eq for MiniVec<T> where T: Eq {}
