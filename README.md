# syscall-table

系统调用(普通函数)的统一抽象和调用。

系统调用（System Call），通常简称为 syscall，是操作系统提供给应用程序的接口之一，用于执行各种特权操作，例如文件操作、进程管理、网络通信、内存管理等。系统调用允许应用程序请求操作系统内核执行特定的任务，而不需要直接操作底层硬件。

通常来说，作为用户态与内核态的边界，系统调用的参数一般是指针或者基本类型，并且参数的数量根据平台的不同而不同。在内核处理系统调用时，会从平台定义的参数寄存器中取出各个参数，同时根据系统调用号调用对应的处理函数，这个过程可能需要逐个对比，如果有一种直接根据调用号一步找到对应处理函数的话就能减少判断的次数，而且在rust中，各个处理函数可能分散在不同的模块中，在处理系统调用时，还需要将这些函数导入到当前作用域中。

`syscall-table`提供了一种方式，可以将这些系统调用的函数实现注册一个新的实现，同时允许用户使用系统调用号或者函数名称直接调用。



## Example

```rust
use syscall_table::{*};

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
impl Into<isize> for SelfUsize {
    fn into(self) -> isize {
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

fn main() {
    invoke_call!(2, 1usize, 1usize);
    invoke_call!(read, 1usize, 2usize);
    let v = invoke_call!(add, 2usize, 4usize);
    assert_eq!(v, 6);
    println!("v = {}", v);
    invoke_call!(test_write, 99usize);
    invoke_call!(5,);

    let point = Point { x: 1, y: 2 };
    let ptr = &point as *const _ as usize;
    invoke_call!(6, ptr);
    let mut point = Point { x: 4, y: 5 };
    let _res = invoke_call!(special_ptr, &mut point);
}

```



## 实现方式

充分利用rust的函数宏和过程宏。