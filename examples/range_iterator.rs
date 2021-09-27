extern crate nd;

use nd::Range;

fn main() {
    let start = [1, 2, 3];
    let stop = [5, 7, 9];
    let step = [1, 2, 1];
    let range = Range { start, stop, step };
    
    println!("Iterating over range: {:?}...", range);
    for index in range.into_iter() {
        println!("{:?}", index);
    }
    println!("Done.");
}
