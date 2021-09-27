use collect_slice::CollectSlice;

use crate::{Range, Size};

pub fn compute_strides<const N: usize>(shape: &Size<N>) -> Size<N> {
    let mut out = [0; N];
    (0..N)
        .map(|i| shape[(i+1)..N].iter().product())
        .collect_slice(&mut out);
    out
}

pub fn ravel_index<const N: usize>(index: &Size<N>, strides: &Size<N>) -> usize {
    index.iter()
        .zip(strides.iter())
        .map(|(i, s)| i*s)
        .sum()
}

pub fn unravel_index<const N: usize>(index: usize, strides: &Size<N>) -> Size<N> {
    let mut idx = index;
    let mut out = [0; N];
    strides.iter()
        .enumerate()
        .map(|(i, s)| {
            if i > 0 { idx %= s }
            idx / s
        })
        .collect_slice(&mut out);
    out
}

pub fn shape_from_bounds<const N: usize>(start: &Size<N>, stop: &Size<N>) -> Size<N> {
    let mut out = [0; N];
    start.iter()
        .zip(stop.iter())
        .map(|(s1, s2)| s2-s1)
        .collect_slice(&mut out);
    out
}

pub fn is_contiguous<const N: usize>(range: &Range<N>, shape: &Size<N>) -> bool {
    let step_size_one = range.step.iter().all(|s| *s == 1);
    if !step_size_one { return false; }
   
    let range_shape = shape_from_bounds(&range.start, &range.stop);
    let all_full_except_last = range_shape.iter()
        .zip(shape.iter())
        .take(N-1)
        .all(|(s1, s2)| s1 == s2);
    let all_one_except_first = range_shape.iter()
        .skip(1)
        .all(|s| *s == 1);

    all_full_except_last || all_one_except_first
}
