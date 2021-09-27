#![feature(generic_associated_types)]
#![feature(trait_alias)]

use std::{
    fmt::{Display, Formatter}, 
    ops::{Index, IndexMut}
};

use collect_slice::CollectSlice;
use num_traits::{Num, Zero};

pub mod index_tricks;

use index_tricks::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    DimensionError(String),
    MemoryError(String)
}

pub type Result<T> = std::result::Result<T, Error>;

/// Mathematical field trait
pub trait Field = Num;

/// N-d Size type
pub type Size<const N: usize> = [usize; N];

pub trait Shape {
    type Output;
    fn shape(&self) -> &Self::Output;
}

pub trait Strides {
    type Output;
    fn strides(&self) -> &Self::Output;
}

pub trait Slice {
    type Range;
    type Output;

    fn slice(&self, range: &Self::Range) -> Self::Output;
}

pub trait View {
    type Range;
    type Output<'a> where Self: 'a;

    fn view<'a>(&'a self, range: &Self::Range) -> Result<Self::Output<'a>>;
}

pub trait Zeros {
    type Shape;
    type Output;

    fn zeros(shape: &Self::Shape) -> Self::Output;
}

pub trait Rand {
    type Shape;
    type Output;

    fn rand(shape: &Self::Shape) -> Self::Output;
}

// TODO: Somehow avoid this level of specificity
pub trait Reshape<const M: usize> {
    type Output;
    fn reshape(self, shape: Size<M>) -> Result<Self::Output>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Range<const N: usize> {
    pub start: Size<N>,
    pub stop: Size<N>,
    pub step: Size<N>
}

#[derive(Clone, Debug, PartialEq)]
pub struct RangeIterator<const N: usize> {
    pub range: Range<N>,
    pub shape: Size<N>,
    pub curr: Size<N>
}

impl<const N: usize> Iterator for RangeIterator<N> {
    type Item = Size<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr[0] >= self.range.stop[0] { return None; }
        let prev = self.curr.clone();

        for i in (0..N).rev() {
            self.curr[i] += self.range.step[i]; 
            if self.curr[i] >= self.range.stop[i] && i > 0 {
                self.curr[i] = self.range.start[i];
            } else {
                break;   
            }
        }

        Some(prev)
    }
}

impl<const N: usize> IntoIterator for Range<N> {
    type Item = Size<N>;
    type IntoIter = RangeIterator<N>;

    fn into_iter(self) -> Self::IntoIter {
        let shape = shape_from_bounds(&self.start, &self.stop);
        let curr = self.start.clone();
        Self::IntoIter {
            range: self,
            shape,
            curr
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Array<K, const N: usize> {
    pub shape: Size<N>,
    pub strides: Size<N>,
    pub data: Box<[K]>
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayView<'a, K, const N: usize> {
    pub shape: Size<N>,
    pub strides: Size<N>,
    pub data: &'a [K]
}

impl<K, const N: usize> Shape for Array<K, N> {
    type Output = Size<N>;

    fn shape(&self) -> &Self::Output {
        &self.shape
    }
}

impl<K, const N: usize> Strides for Array<K, N> {
    type Output = Size<N>;

    fn strides(&self) -> &Self::Output {
        &self.strides
    }
}

impl<K, const N: usize> Zeros for Array<K, N>
where
    K: Zero + Clone
{
    type Shape = Size<N>;
    type Output = Self;

    fn zeros(shape: &Self::Shape) -> Self::Output {
        Self {
            shape: shape.clone(),
            strides: compute_strides(shape),
            data: vec![K::zero(); shape.iter().product()].into_boxed_slice()
        }
    }
}

impl<K, const N: usize> Index<Size<N>> for Array<K, N> {
    type Output = K;

    fn index(&self, index: Size<N>) -> &Self::Output {
        &self.data[ravel_index(&index, &self.strides)]
    }
}

impl<K, const N: usize> IndexMut<Size<N>> for Array<K, N> {
    fn index_mut(&mut self, index: Size<N>) -> &mut Self::Output {
        &mut self.data[ravel_index(&index, &self.strides)]
    }
}

impl<K, const N: usize> Slice for Array<K, N>
where
    K: Copy + Zero
{
    type Range = Range<N>;
    type Output = Self;

    fn slice(&self, range: &Self::Range) -> Self::Output {
        let shape = shape_from_bounds(&range.start, &range.stop);
        let strides = compute_strides(&shape);
        let mut data = vec![K::zero(); shape.iter().product()].into_boxed_slice();

        range.clone().into_iter()
            .map(|index| self[index])
            .collect_slice(&mut data);

        Self { shape, strides, data }
    }
}

impl<K, const N: usize> View for Array<K, N> {
    type Range = Range<N>;
    type Output<'a> where Self: 'a = ArrayView<'a, K, N>;
    
    fn view<'a>(&'a self, range: &Self::Range) -> Result<Self::Output<'a>> {
        if !is_contiguous(range, &self.shape) {
            return Err(Error::MemoryError(format!("Range {:?} is not contiguous within array of shape {:?}", range, self.shape)));
        }
        
        let shape = shape_from_bounds(&range.start, &range.stop);
        let strides = compute_strides(&shape);
        
        let start_flat = ravel_index(&range.start, &strides);
        let stop_flat = start_flat + shape.iter().product::<usize>();
        let data = &self.data[start_flat..stop_flat];

        Ok(Self::Output::<'a> { shape, strides, data }) 
    }
}

impl<'b, K, const N: usize> View for ArrayView<'b, K, N> {
    type Range = Range<N>;
    type Output<'a> where Self: 'a = ArrayView<'a, K, N>;
    
    fn view<'a>(&'a self, range: &Self::Range) -> Result<Self::Output<'a>> {
        if !is_contiguous(range, &self.shape) {
            return Err(Error::MemoryError(format!("Range {:?} is not contiguous within array of shape {:?}", range, self.shape)));
        }
        
        let shape = shape_from_bounds(&range.start, &range.stop);
        let strides = compute_strides(&shape);
        
        let start_flat = ravel_index(&range.start, &strides);
        let stop_flat = start_flat + shape.iter().product::<usize>();
        let data = &self.data[start_flat..stop_flat];

        Ok(Self::Output::<'a> { shape, strides, data }) 
    }
}

impl<K, const N: usize, const M: usize> Reshape<M> for Array<K, N> {
    type Output = Array<K, M>;

    fn reshape(self, shape: Size<M>) -> Result<Self::Output> {
        let p1: usize = self.shape.iter().product();
        let p2: usize = shape.iter().product();

        if p1 != p2 {
            return Err(Error::DimensionError(format!("Shape products do not match for {:?} and {:?}", self.shape, shape)));
        }

        let strides = compute_strides(&shape);
        Ok(Self::Output { shape, strides, data: self.data })
    }
}

impl<'a, K, const N: usize, const M: usize> Reshape<M> for ArrayView<'a, K, N> {
    type Output = ArrayView<'a, K, M>;

    fn reshape(self, shape: Size<M>) -> Result<Self::Output> {
        let p1: usize = self.shape.iter().product();
        let p2: usize = shape.iter().product();

        if p1 != p2 {
            return Err(Error::DimensionError(format!("Shape products do not match for {:?} and {:?}", self.shape, shape)));
        }

        let strides = compute_strides(&shape);
        Ok(Self::Output { shape, strides, data: self.data })
    }
}

fn format_helper<'a, K, const N: usize>(view: &ArrayView<'a, K, N>, d: usize, i: &mut usize, builder: &mut string_builder::Builder)
where
    K: Display
{
    let prefix = " ".repeat(d);
    if d < N-1 {
        builder.append(format!("{}[\n", prefix));
        for j in 0..view.shape[d] {
            format_helper(view, d+1, i, builder);
            builder.append("\n");
        }
        builder.append(format!("{}]\n", prefix));
    } else {
        builder.append(format!("{}[", prefix));
        for j in 0..view.shape[d] {
            if j < view.shape[d]-1 {
                builder.append(format!("{}, ", view.data[*i]));
            } else {
                builder.append(format!("{}]", view.data[*i]));
            }
            *i += 1;
        }
    }
}

impl<'a, K, const N: usize> Display for ArrayView<'a, K, N>
where
    K: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut builder = string_builder::Builder::default();
        let mut i = 0;
        format_helper(self, 0, &mut i, &mut builder);
        write!(f, "{}", builder.string().unwrap())
    }
}
