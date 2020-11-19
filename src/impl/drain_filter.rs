pub struct DrainFilter<'a, T, F>
where
    F: core::ops::FnMut(&mut T) -> bool,
{
    vec: &'a mut crate::MiniVec<T>,
    pred: F,
}
