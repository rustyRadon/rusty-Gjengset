fn main() {
    println!("Hello, world!");
}

//rules around ownership, move and copy semantics, and dropping.
let x1 = 42;
let y1 = Box::new(84);
{ // starts a new scope
1 let z = (x1, y1);
// z goes out of scope, and is dropped;
// it in turn drops the values from x1 and y1
2 }
// x1's value is Copy, so it was not moved into z
3 let x2 = x1;
// y1's value is not Copy, so it was moved into z
4 // let y2 = y1;

fn noalias(input: &i32, output: &mut i32) {
if *input == 1 {
1 *output = 2;
}
2 if *input != 1 {
*output = 3;
}
}

let x = 42;
let mut y = &x; // y is of type &i32        // you are able to change the value of the pointer y to a different value
let z = &mut y; // z is of type &mut &i32