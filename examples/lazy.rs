extern crate core;

use syscall_table::ServiceWrapper;
use systable_macro_derive::syscall_func;

#[syscall_func(3)]
fn add(a: usize, b: usize) -> isize {
    println!("add {} + {}", a, b);
    (a + b) as isize
}

fn fake(_x: &[usize]) -> isize {
    println!("fake");
    0
}
struct Flag {
    short: char,
    name: &'static str,
}

impl Flag {
    const fn new(short: char, name: &'static str) -> Self {
        Flag { short, name }
    }
}

inventory::submit! {
    Flag::new('v', "verbose")
}

inventory::collect!(Flag);

inventory::submit!(ServiceWrapper {
    service: fake,
    id: 3,
});

fn main() {
    for flag in inventory::iter::<Flag> {
        println!("-{}, --{}", flag.short, flag.name);
    }
    for wrapper in inventory::iter::<ServiceWrapper> {
        println!("id = {}", wrapper.id);
    }
}
