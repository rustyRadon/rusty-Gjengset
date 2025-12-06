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


let x = 42;
let mut y = &x; // y is of type &i32        // you are able to change the value of the pointer y to a different value
let z = &mut y; // z is of type &mut &i32

fn replace_with_84(s: &mut Box<i32>) {
// this is not okay, as *s would be empty:
1 // let was = *s;
// but this is:
2 let was = std::mem::take(s);   //Creates a new Box<i32>::default() (which is Box::new(0) for i32)
// so is this:
3 *s = was;
// we can exchange values behind &mut:
let mut r = Box::new(84);
4 std::mem::swap(s, &mut r);   
assert_ne!(*r, 84);
}
let mut s = Box::new(42);
replace_with_84(&mut s);

//...........................std::mem::take() - Replace with Default
// Before: s → Box(42)

// During take():
// 1. Create: Box(0) [temporary]
// 2. Swap: s → Box(0), was → Box(42)
// 3. Return: was = Box(42)

// After: s → Box(0)  // STILL A VALID BOX!
//........................... Direct Assignment - Replace with Another Value

// s has Box(0)
// was has Box(42)
// *s = was: 
//   - Box(0) dropped
//   - s now points to Box(42)

//............................std::mem::swap() - Exchange Values
// Before:
//   s → Box(42)
//   r → Box(84)

// After swap:
//   s → Box(84)  // Valid!
//   r → Box(42)  // Valid!

//..............................THREE SAFE PATTERNS
//Pattern 1: Take and Leave Default
use std::mem;

fn take_value(s: &mut String) -> String {
    mem::take(s)  // Leaves empty string ""
}

let mut my_string = String::from("hello");
let old = take_value(&mut my_string);
// my_string is now "" (valid empty string)
// old is "hello"

///////Pattern 2: Replace with Specific Value
fn replace_value(s: &mut String, new_value: String) -> String {
    mem::replace(s, new_value)
}

let mut s = String::from("hello");
let old = replace_value(&mut s, String::from("world"));
// s is now "world", old is "hello"

//////Pattern 3: Swap Between Two Refs
fn swap_values(a: &mut String, b: &mut String) {
    mem::swap(a, b);
}

let mut x = String::from("foo");
let mut y = String::from("bar");
swap_values(&mut x, &mut y);
// x is "bar", y is "foo"

//.............................Interior Mutability: Breaking the Rules Safely
let x = 42;
let shared_ref = &x;
// shared_ref.push(5);  // ❌ Can't mutate through shared reference!

///////////////Category 1: Types that give you &mut T through &T
//enforce borrowing rules at runtime instead of compile-time:

//..Mutex<T> (for threads)
use std::sync:Mutex;

let counter = Mutex::new(0);   //shared across threads
let shared_ref = &counter;

{
    let mut lock = shared_ref.lock().unwrap();
    *lock += 1;  // Now have &mut access!
} // Lock released automatically

//..RefCell<T> (for single-threaded)
use std::cell::RefCell;

let cell = RefCell::new(42);
let shared_ref = &cell;

{
    let mut borrow = shared_ref.borrow_mut();  // Runtime check!
    *borrow += 1;
}  // Borrow released

// Will panic at runtime if:
// let b1 = cell.borrow_mut();
// let b2 = cell.borrow_mut();  // ❌ Already borrowed mutably!

/////visuals
// RefCell { value: 42, borrow_count: 0 }

// borrow_mut() → RefCell { value: 42, borrow_count: MUTABLE }
//     Can't borrow() or borrow_mut() again until released

// borrow() → RefCell { value: 42, borrow_count: SHARED(1) }
//     Can borrow() again, but not borrow_mut()
 
////////////Category 2: Types that let you replace values (no &mut given)
//Get a copy of the value and Replace the entire value

//Cell<T> (single-threaded)
use std::cell::Cell;

let cell = Cell::new(42);
let shared_ref = &cell;

// No references to inner value!
let current = cell.get();  // Copies the value
cell.set(100);             // Replaces the value
// cell.get_mut()          // ❌ Doesn't exist!

//Atomic Types (thread-safe)
use std::sync::atomic::{AtomicUsize, Ordering};

let counter = AtomicUsize::new(0);
let shared_ref = &counter;

shared_ref.fetch_add(1, Ordering::SeqCst);  // Thread-safe increment
let value = shared_ref.load(Ordering::SeqCst);  // Get copy

///////////The Magic Ingredient: UnsafeCell
pub struct UnsafeCell<T> {
    value: T,
}

impl<T> UnsafeCell<T> {
    pub fn get(&self) -> *mut T {
        &self.value as *const T as *mut T
    }
}    //UnsafeCell is the only way in Rust to mutate through a shared reference. The compiler treats it specially.

///// Real-World Examples
//Example 1: Caching (with RefCell)
use std::cell::RefCell;

struct CachedCalculator {
    cache: RefCell<HashMap<i32, i32>>,   //cache — stores computed values RefCell<HashMap<...>> allows mutation of the cache even when we only have &self.
    computation_count: Cell<u32>,        //computation_count — counts how many times a computation actually happens Cell<u32> lets you set/get values with interior mutability.
}

impl CachedCalculator {
    fn compute(&self, x: i32) -> i32 {
        // Can read cache through &self!
        if let Some(result) = self.cache.borrow().get(&x) {   //self.cache.borrow() creates a runtime-checked immutable borrow. If the result already exists in the cache, return it.
            return *result;
        }
        
        let result = expensive_computation(x);
        self.cache.borrow_mut().insert(x, result);                     //borrow_mut() gives a mutable borrow even though we have &self. Inserts computed value into the cache.
        self.computation_count.set(self.computation_count.get() + 1);  //Cell allows reading (get()) and writing (set()) a value without &mut self. 
        result
    }
}

//Example 2: Thread-safe Counter (with Atomic)
use std::sync::atomic::AtomicUsize;                            //AtomicUsize — thread-safe integer supporting atomic operations.
use std::sync::Arc;                                            //reference-counted smart pointer so you can share data across threads.

let counter = Arc::new(AtomicUsize::new(0));                   //Creates a shared atomic counter with value 0. Arc is necessary because the counter will be shared across threads.

let mut threads = vec![];                                      //Vector to store thread handles.
for _ in 0..10 {                                               //Loop 10 times to spawn 10 threads.
    let counter = Arc::clone(&counter);                        //Clones the Arc pointer (cheap), so each thread gets a reference to the same atomic counter.
    threads.push(std::thread::spawn(move || {                  //Spawns a new thread and moves the cloned Arc into the thread.
        counter.fetch_add(1, Ordering::SeqCst);                //Atomically increments the counter. Ordering::SeqCst is the strictest memory ordering.
    }));
}

//////////////////////easy way to understand
//Example Without Sharing

//If you did:
let x = 0;


//and tried to move x into two threads:

let x = 0;
let a = std::thread::spawn(|| println!("{x}"));
let b = std::thread::spawn(|| println!("{x}"));

// This does NOT work.

// Reason:
// Each value in Rust can only have one owner, and you can’t move the same non-Copy value into two threads.
// The threads would each need their own copy, not shared access.


//Example With Sharing

//When you do this:
let counter = Arc::new(AtomicUsize::new(0));

//it means the same counter will be accessed by all threads:

// Thread A → increments same counter
// Thread B → increments same counter
// Thread C → increments same counter
// ...

// All threads modify one shared integer, not ten separate integers.

//When Thread 2 runs:

counter += 1
//all other threads will see that the counter changed.

// Good for small, copyable data
let point = Cell::new((0, 0));
point.set((1, 2));

// Good for complex mutation in single thread
let shared_data = Rc::new(RefCell::new(Vec::new()));
shared_data.borrow_mut().push(42);

// Good for thread-safe counters
let atomic_counter = AtomicUsize::new(0);

//................................LIFETIME
//Lifetime with holes
let mut x = Box::new(42);
1 let mut z = &x;     // Lifetime 'a starts
// 'a

for i in 0..100 {
2   println!("{}", z); // Use of z
   // 'a
    
3   x = Box::new(i);   // Move x! (mutably borrows x)
4   z = &x;            // Reassign z
   // 'a restarts
}

println!("{}", z);     // Use of z
// 'a

// The "Hole" in the Lifetime:

// Loop iteration:
// Start: z points to x (valid)
// 2: Use z (valid)
// 3: MOVE x (invalidates current z!)
//    └─ Lifetime 'a ENDS here
// 4: Reassign z = &x (new lifetime starts)
// Repeat...

// Visual Timeline:

// Iteration 1:
// 1: z₀ = &x₀ (lifetime L₀ starts)
// 2: Use z₀ (valid)
// 3: Move x₀ → x₁ (L₀ ends, z₀ invalidated)
// 4: z₁ = &x₁ (lifetime L₁ starts)

// Iteration 2:
// 2: Use z₁ (valid)
// 3: Move x₁ → x₂ (L₁ ends, z₁ invalidated)
// 4: z₂ = &x₂ (lifetime L₂ starts)

//Programmers naturally write code like:

let ref = &data;
if condition {
    use(ref);
} else {
    // Don't use ref here
    mutate(&mut data);  // Should be OK!
}

//..some cses
use std::mem;

fn split_at_mut(slice: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = slice.len();
    let ptr = slice.as_mut_ptr();
    
    // Borrow checker: "Two &mut to same slice? ERROR!"
    // But we know they're to different parts...
    
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

//WORKAROUND
// Problem: Reference lives too long
let r = &data;
mutate(&mut data);  // ❌
use(r);

// Solution 1: Limit scope
{
    let r = &data;
    use(r);
}  // r's lifetime ends
mutate(&mut data);  // ✅

// Solution 2: Restructure
if need_to_mutate {
    mutate(&mut data);
} else {
    let r = &data;
    use(r);
}

//...........................GENERIC LIFETIME
//ometimes you want a struct that holds references instead of owning data:
// Instead of owning strings:
struct OwnedSplitter {
    document: String,    // Owns - allocates memory
    delimiter: String,   // Owns - allocates memory
}

// You can store references:
struct RefSplitter<'a> {
    document: &'a str,   // Borrows - no allocation
    delimiter: &'a str,   // Borrows - no allocation
}

//Basic generic life time syntax
// Single lifetime parameter
struct MyStruct<'a> {
    data: &'a str,
}

// Multiple lifetime parameters
struct TwoRefs<'a, 'b> {
    first: &'a str,
    second: &'b str,
}

// Lifetime parameters with type parameters
struct Mixed<'a, T> {
    reference: &'a T,
    owned: T,
}

//////..Key Insight 1: Drop Changes Lifetime Rules
//Without Drop:

struct Holder<'a> {
    data: &'a str,
}

impl<'a> Holder<'a> {
    fn get(&self) -> &'a str {
        self.data
    }
}

fn example() {
    let string = String::from("hello");
    let holder;
    {
        let borrowed = &string;
        holder = Holder { data: borrowed };
        // holder can be dropped anytime after this point
        // because it doesn't implement Drop
    }  // borrowed lifetime ends here
    // holder still exists but can't be used (no valid reference)
    // This is OK because Holder doesn't use the reference when dropped
}

//with drop
struct DropHolder<'a> {
    data: &'a str,
}

impl<'a> Drop for DropHolder<'a> {           //When a DropHolder goes out of scope, Rust will automatically run this drop function. This function uses self.data (the reference).
    fn drop(&mut self) {
        // Might use self.data here!
        println!("Dropping: {}", self.data);
    }
}

fn example() {
    let string = String::from("hello");
    let holder;
    {
        let borrowed = &string;
        holder = DropHolder { data: borrowed };
    }  // borrowed lifetime ends here
    // ❌ ERROR: holder is dropped here, but its data reference is invalid!
    // Drop::drop might try to use the invalid reference
}

struct StrSplit<'s, 'p> {
delimiter: &'p str,
document: &'s str,
}
impl<'s, 'p> Iterator for StrSplit<'s, 'p> {
type Item = &'s str;
fn next(&self) -> Option<Self::Item> {
todo!()
}
}
fn str_before(s: &str, c: char) -> Option<&str> {
StrSplit { document: s, delimiter: &c.to_string() }.next()


///////////////////
struct MutStr<'a, 'b> {
s: &'a mut &'b str
}
let mut s = "hello";
1 *MutStr { s: &mut s }.s = "world";
println!("{}", s);
Listing 1-11: A type that needs to be generic over multiple lifetimes



