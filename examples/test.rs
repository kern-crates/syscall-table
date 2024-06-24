use syscall_table::*;

#[syscall_func(1)]
fn test_write(p: usize) -> isize {
    println!("test_write {}", p);
    0
}
#[syscall_func(2)]
fn read(p1: usize, p2: usize) -> isize {
    println!("p1+p2 = {}", p1 + p2);
    (p1 + p2) as isize
}

#[syscall_func(4)]
fn test(p1: usize, p2: usize, p3: *const u8) -> isize {
    let len = p1 + p2;
    let buf = unsafe { core::slice::from_raw_parts(p3, len) };
    let buf = buf
        .chunks(8)
        .map(|x| {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(x);
            usize::from_le_bytes(buf)
        })
        .collect::<Vec<usize>>();
    println!("read {}, buf = {:?}", len, buf);
    0
}

struct SelfUsize(usize);
impl From<usize> for SelfUsize {
    fn from(value: usize) -> Self {
        SelfUsize(value)
    }
}

impl ToIsize for SelfUsize {
    fn to_isize(self) -> isize {
        self.0 as isize
    }
}

#[syscall_func(3)]
fn add(a: usize, b: usize) -> SelfUsize {
    (a + b).into()
}

#[syscall_func(5)]
fn empty_arg() -> isize {
    println!("empty_arg");
    0
}

struct Point {
    x: usize,
    y: usize,
}

#[syscall_func(6)]
fn special_ptr(point: *const Point) -> isize {
    let point = unsafe { point.as_ref() }.unwrap();
    println!("special_ptr x = {}, y = {}", point.x, point.y);
    0
}

#[syscall_func(7)]
fn result_fn() -> Result<(), i32> {
    Err(-2)
}


#[syscall_func(8)]
fn result_fn2() -> Result<usize, i32> {
    Ok(1)
}

fn main() {
    let mut table = Table::new();
    register_syscall!(table, (0, read), (1, test));
    table.do_call(0, &[1, 2]);
    let data = [6usize; 8];
    table.do_call(1, &[0, 8 * 8, data.as_ptr() as usize]);
    table.register(2, test_write);


    println!("invoke_call:");
    invoke_call!(read, 1usize, 2usize);
    let v = invoke_call!(add, 2usize, 4usize);
    assert_eq!(v, 6);
    println!("v = {}", v);
    invoke_call!(test_write, 99usize);

    let mut point = Point { x: 4, y: 5 };
    let _res = invoke_call!(special_ptr, &mut point);
    for wrapper in inventory::iter::<ServiceWrapper> {
        println!("id = {}", wrapper.id);
    }

    let r = invoke_call_id!(3, 1usize, 2usize);
    assert_eq!(r,3);
    println!("r = {}", r);
    invoke_call_id!(5,);

    let res = invoke_call!(result_fn,);
    assert_eq!(res, -2);

    let res = invoke_call_id!(8,);
    assert_eq!(res,1)
}
