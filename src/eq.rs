use crate::MiniVec;

extern crate core;

use core::cmp::Eq;

impl<T> Eq for MiniVec<T> where T: Eq {}
