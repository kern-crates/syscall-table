# syscall-table

系统调用(普通函数)的统一抽象和调用。

系统调用（System Call），通常简称为 syscall，是操作系统提供给应用程序的接口之一，用于执行各种特权操作，例如文件操作、进程管理、网络通信、内存管理等。系统调用允许应用程序请求操作系统内核执行特定的任务，而不需要直接操作底层硬件。

通常来说，作为用户态与内核态的边界，系统调用的参数一般是指针或者基本类型，并且参数的数量根据平台的不同而不同。在内核处理系统调用时，会从平台定义的参数寄存器中取出各个参数，同时根据系统调用号调用对应的处理函数。

`syscall-table`提供了一种方式，可以将这些系统调用的函数实现注册一个新的实现，同时允许用户使用系统调用号或者函数名称直接调用。这样就不需要大段的`match`了。



## Example

```rust
use syscall_table::*;

#[syscall_func(1)]
fn test_write(p: usize) -> isize {
    println!("test_write {}", p);
    0
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
#[syscall_func(7)]
fn result_fn() -> Result<(), i32> {
    Err(-2)
}
#[syscall_func(8)]
fn result_fn2() -> Result<usize, i32> {
    Ok(1)
}
fn main() {
    println!("invoke_call:");
    let v =  invoke_call!(test_write, 1usize);
    assert_eq!(v, 0);
    invoke_call!(test_write, 99usize);
    let mut point = Point { x: 4, y: 5 };
    let _res = invoke_call!(special_ptr, &mut point);
    let r = invoke_call_id!(3, 1usize, 2usize);
    println!("r = {}", r);
    invoke_call_id!(5,);
    let res = invoke_call!(result_fn,);
    assert_eq!(res, -2);

    let res = invoke_call_id!(8,);
    assert_eq!(res,1)
}

```



## 实现方式

1. 利用rust的函数宏和过程宏定义函数的统一形式

2. 使用`inventory` 收集所有的实现

   1. 其利用`.init_array`段，这是一个特殊的段，在用户态，libc会调用这里面的函数指针，完成一些特定动作，在内核态，我们需要在使用前手动去调用这个段中的初始化函数。
   2. 库中提供了一个宏`init_init_array`完成这个事情。请在链接脚本中定义如下这个段以便可以找到其所在位置。这个段通常位于.data段中

   ```
   sinit = .;
   *(.init_array .init_array.*)
   einit = .;
   ```

3. `invoke_call_id` 遍历收集到的实现，找到对应id的实现

4. `invoke_call`会直接调用而不需要查找



## TODO

`invoke_call_id` 这个过程仍然需要逐个对比，如果有一种直接根据调用号一步找到对应处理函数的话就能减少判断的次数。

思路：以空间换时间