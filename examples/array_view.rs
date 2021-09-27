extern crate nd;

use nd::{Array, Range, View, Zeros};

fn main() {
    let shape = [3, 4, 5];
    let mut array: Array<f32, 3> = Array::zeros(&shape);
    for i in 0..shape.iter().product() {
        array.data[i] = i as f32;
    }

    let start = [0, 0, 0];
    let stop = shape.clone();
    let step = [1, 1, 1];
    let range = Range { start, stop, step };

    println!("{}", array.view(&range).unwrap());
}
