//! syscall table
//!
//! Since the number of parameters used by each system call is inconsistent,
//! we need to pay attention to the use of parameters when calling the system call.
//! This module uses the macro mechanism of rust to abstract the various system calls.
//! Users only need to register the system call number and function To a global table,
//! when in use, directly pass the system call number and a `&[usize]` parameter array.
//!
//! # Example
//! ```
//!use syscall_table::Table;
//! fn read(p1: usize, p2: usize) -> isize {
//!     println!("p1+p2 = {}", p1 + p2);
//!     0
//! }
//! fn test(p1: usize, p2: usize, p3: *const u8) -> isize {
//!     let len = p1 + p2;
//!     let buf = unsafe { core::slice::from_raw_parts(p3, len) };
//!     // transfer to usize
//!     let buf = buf
//!         .chunks(8)
//!         .map(|x| {
//!             let mut buf = [0u8; 8];
//!             buf.copy_from_slice(x);
//!             usize::from_le_bytes(buf)
//!         })
//!         .collect::<Vec<usize>>();
//!     println!("read {}, buf = {:?}", len, buf);
//!     0
//! }
//! let mut table = Table::new();
//! table.register(0, read);
//! table.register(1, test);
//! table.do_call(0, &[1, 2, 0, 0, 0, 0]);
//! let data = [6usize; 8];
//! table.do_call(1, &[0, 8 * 8, data.as_ptr() as usize, 0, 0, 0]);
//!```

#![cfg_attr(not(feature = "test"), no_std)]
#![allow(non_snake_case)]
#![deny(missing_docs)]
extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
pub use inventory::{iter, submit};
pub use paste::paste;
pub use systable_macro_derive::syscall_func;
/// Uniform function
pub trait UniFn<Args, Res> {
    /// Call the function
    fn call(&self, args: Args) -> Res;
}

macro_rules! unifn_tuple {
    ($(($arg:ident,$n:tt)),+) => {
        impl<T,$($arg,)+ Res> UniFn<($($arg,)+),Res> for T
        where
            T: Fn($($arg,)+)->Res
        {
            fn call(&self,args:($($arg,)+))->Res{
                (self)($(args.$n,)+)
            }
        }
    };
}
impl<T, Res> UniFn<(), Res> for T
where
    T: Fn() -> Res,
{
    fn call(&self, _: ()) -> Res {
        (self)()
    }
}
unifn_tuple!((P0, 0));
unifn_tuple!((P0, 0), (P1, 1));
unifn_tuple!((P0, 0), (P1, 1), (P2, 2));
unifn_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3));
unifn_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3), (P4, 4));
unifn_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3), (P4, 4), (P5, 5));
unifn_tuple!(
    (P0, 0),
    (P1, 1),
    (P2, 2),
    (P3, 3),
    (P4, 4),
    (P5, 5),
    (P6, 6)
);

/// A wrapper of uniform function
#[derive(Copy, Clone)]
pub struct SysCallHandler<F, Args, Res> {
    func: F,
    _args: core::marker::PhantomData<Args>,
    _res: core::marker::PhantomData<Res>,
}

impl<F, Args, Res> SysCallHandler<F, Args, Res>
where
    F: UniFn<Args, Res>,
{
    /// Create a new SysCallHandler
    pub const fn new(func: F) -> Self {
        Self {
            func,
            _args: core::marker::PhantomData,
            _res: core::marker::PhantomData,
        }
    }
    /// Call the function
    pub fn call(&self, args: Args) -> Res {
        self.func.call(args)
    }
}

/// Trait for converting to isize
pub trait ToIsize {
    /// Convert to isize
    fn to_isize(self) -> isize;
}

macro_rules! mark_to_isize {
    ($ident:ty) => {
        impl ToIsize for $ident {
            fn to_isize(self) -> isize {
                self as isize
            }
        }
    };
}
impl ToIsize for (){
    fn to_isize(self) -> isize {
        0
    }
}
mark_to_isize!(usize);
mark_to_isize!(u64);
mark_to_isize!(u32);
mark_to_isize!(u16);
mark_to_isize!(u8);
mark_to_isize!(isize);
mark_to_isize!(i64);
mark_to_isize!(i32);
mark_to_isize!(i16);
mark_to_isize!(i8);

impl<T:ToIsize, E: ToIsize> ToIsize for Result<T, E> {
    fn to_isize(self) -> isize {
        match self {
            Ok(t) => t.to_isize(),
            Err(e) => e.to_isize(),
        }
    }
}

/// Trait for converting to usize
pub trait ToUsize {
    /// Convert to usize
    fn to_usize(self) -> usize;
}

macro_rules! mark_to_usize {
    ($ident:ty) => {
        impl ToUsize for $ident {
            fn to_usize(self) -> usize {
                self as usize
            }
        }
    };
}
mark_to_usize!(usize);
mark_to_usize!(u64);
mark_to_usize!(u32);
mark_to_usize!(u16);
mark_to_usize!(u8);
mark_to_usize!(isize);
mark_to_usize!(i64);
mark_to_usize!(i32);
mark_to_usize!(i16);
mark_to_usize!(i8);

impl<T> ToUsize for *const T {
    fn to_usize(self) -> usize {
        self as usize
    }
}
impl<T> ToUsize for *mut T {
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl ToUsize for () {
    fn to_usize(self) -> usize {
        0
    }
}

impl<T> ToUsize for &T {
    fn to_usize(self) -> usize {
        self as *const T as usize
    }
}

impl<T> ToUsize for &mut T {
    fn to_usize(self) -> usize {
        self as *mut T as usize
    }
}

/// Trait for converting arguments
pub trait FromArgs: Sized {
    /// Convert arguments
    fn from(args: &[usize]) -> Result<Self, String>;
}

impl FromArgs for () {
    fn from(_: &[usize]) -> Result<Self, String> {
        Ok(())
    }
}

impl<T> FromArgs for *const T {
    fn from(args: &[usize]) -> Result<Self, String> {
        if args.len() >= 1 {
            let res = args[0] as *const T;
            Ok(res)
        } else {
            Err(alloc::format!("{}:args.len() < 1", stringify!($ident)))
        }
    }
}

impl<T> FromArgs for *mut T {
    fn from(args: &[usize]) -> Result<Self, String> {
        if args.len() >= 1 {
            let res = args[0] as *mut T;
            Ok(res)
        } else {
            Err(alloc::format!("{}:args.len() < 1", stringify!($ident)))
        }
    }
}

macro_rules! mark_basic_type {
    ($ident:ty) => {
        impl FromArgs for $ident {
            fn from(args: &[usize]) -> Result<Self, String> {
                if args.len() >= 1 {
                    let res = args[0] as $ident;
                    Ok(res)
                } else {
                    Err(crate::alloc::format!(
                        "{}:args.len() < 1",
                        stringify!($ident)
                    ))
                }
            }
        }
    };
}
mark_basic_type!(usize);
mark_basic_type!(u64);
mark_basic_type!(u32);
mark_basic_type!(u16);
mark_basic_type!(u8);
mark_basic_type!(isize);
mark_basic_type!(i64);
mark_basic_type!(i32);
mark_basic_type!(i16);
mark_basic_type!(i8);

macro_rules! from_args_tuple {
    ($(($arg:ident,$n:tt)),+) => {
        impl<$($arg,)+> FromArgs for ($($arg,)+)
        where
            $($arg:FromArgs,)+
        {
            fn from(args:&[usize])->Result<Self,String>{
                $(let $arg = $arg::from(&args[$n..])?;)+
                Ok(($($arg,)+))
            }
        }
    };
}

from_args_tuple!((P0, 0));
from_args_tuple!((P0, 0), (P1, 1));
from_args_tuple!((P0, 0), (P1, 1), (P2, 2));
from_args_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3));
from_args_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3), (P4, 4));
from_args_tuple!((P0, 0), (P1, 1), (P2, 2), (P3, 3), (P4, 4), (P5, 5));

/// The wrapper of syscall handler
pub struct Service {
    /// The handler of syscall
    service: Box<dyn Fn(&[usize]) -> isize>,
}

impl Service {
    /// Create a new Service
    ///
    /// The SysCallHandler will be put into a closure, thus erasing the function parameter information
    pub fn from_handler<F, Args, Res>(handler: SysCallHandler<F, Args, Res>) -> Self
    where
        F: UniFn<Args, Res> + 'static,
        Args: FromArgs + 'static,
        Res: ToIsize + 'static,
    {
        Self {
            service: Box::new(move |args: &[usize]| {
                let args = Args::from(args).unwrap();
                handler.call(args).to_isize()
            }),
        }
    }
    /// Call the service
    pub fn handle(&self, args: &[usize]) -> isize {
        (self.service)(args)
    }
}

unsafe impl Send for Service {}
unsafe impl Sync for Service {}

/// A container for Service
///
/// The key is the specific number
pub struct Table {
    map: BTreeMap<usize, Service>,
}

impl Table {
    /// Create a new Table
    pub const fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    /// Register a function
    pub fn register<F, Args, Res>(&mut self, id: usize, func: F)
    where
        F: UniFn<Args, Res> + 'static,
        Args: FromArgs + 'static,
        Res: ToIsize + 'static,
    {
        let handler = SysCallHandler::new(func);
        self.map.insert(id, Service::from_handler(handler));
    }
    /// Remove a function
    pub fn remove(&mut self, id: usize) -> Option<Service> {
        self.map.remove(&id)
    }

    /// call the function
    pub fn do_call(&self, id: usize, args: &[usize]) -> Option<isize> {
        self.map.get(&id).map(|x| x.handle(args))
    }
}

/// Register one or more functions
///
/// # Example
/// ```
/// use syscall_table::{register_syscall, Table};
/// fn read(p1: usize, p2: usize) -> isize {
///   println!("p1+p2 = {}", p1 + p2);
///  0
/// }
/// fn add(a: usize, b: usize) -> isize {
///    (a + b) as isize
/// }
/// let mut table = Table::new();
/// register_syscall!(table, (0, read), (1, add));
/// let v = table.do_call(1, &[2, 4]);
/// assert_eq!(v, Some(6));
/// ```
///
#[macro_export]
macro_rules! register_syscall {
    ($table:ident,$(($id:expr,$func:ident)),+ $(,)?) => {
        $(
            $table.register($id,$func);
        )+
    };
}

/// Create a SysCallHandler
pub const fn register<F, Args, Res>(func: F) -> SysCallHandler<F, Args, Res>
where
    F: UniFn<Args, Res> + 'static,
    Args: FromArgs + 'static,
    Res: ToIsize + 'static,
{
    let handler = SysCallHandler::new(func);
    handler
}

/// Call the function according to the name
///
/// # Example
/// ```compile_fail
/// use syscall_table::{invoke_call, syscall_func,ToUsize};
/// #[syscall_func(1)]
/// fn add(p1: usize,p2:usize) -> isize {
///     p1 as isize + p2 as isize
/// }
/// let res = invoke_call!(1,1usize,1usize);
/// assert_eq!(res,2);
/// let res = invoke_call!(add,1usize,1usize);
/// assert_eq!(res,2);
/// ```
///
#[macro_export]
macro_rules! invoke_call {
     ($name:ident,$($arg:expr),* $(,)?) => {
        {
            let p = [$(ToUsize::to_usize($arg)),*];
            crate::paste!{
                extern "C" {
                    fn [<  __ $name >](p:&[usize])->isize;
                }
                let res:isize = unsafe{[< __ $name >](&p)};
                res
            }
        }
    };

}

/// For inventory
pub struct ServiceWrapper {
    /// The service
    pub service: fn(&[usize]) -> isize,
    /// The id
    pub id: u16,
}

inventory::collect!(ServiceWrapper);

/// Call the function according to the syscall number
#[macro_export]
macro_rules! invoke_call_id {
    ($number:expr,$($arg:expr),* $(,)?) => {
        {
            use crate::{iter,ServiceWrapper,ToUsize};
            #[inline]
            fn search_run(id:u16,p:&[usize])->isize{
                for wrapper in iter::<ServiceWrapper> {
                    if id == wrapper.id {
                        return (wrapper.service)(&p);
                    }
                }
                panic!("can't find id = {}",id);
            }
            let p = [$(ToUsize::to_usize($arg)),*];
            let id = $number as u16;
            search_run(id,&p)
        }
    };
}

/// The User should call this macro to call the init function in .init_array section
#[macro_export]
macro_rules! init_init_array {
    () => {
        extern "C" {
            fn sinit();
            fn einit();
        }
        unsafe {
            let fn_array = core::slice::from_raw_parts(
                sinit as usize as *mut usize,
                (einit as usize - sinit as usize) / 8,
            );
            for fn_ptr in fn_array {
                let fn_ptr = core::mem::transmute::<usize, fn() -> isize>(*fn_ptr);
                fn_ptr();
            }
        }
    };
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::Table;
    use std::println;
    use std::vec::Vec;
    fn read(p1: usize, p2: usize) -> isize {
        println!("p1+p2 = {}", p1 + p2);
        0
    }

    fn add(a: usize, b: usize) -> isize {
        (a + b) as isize
    }
    fn test(p1: usize, p2: usize, p3: *const u8) -> isize {
        let len = p1 + p2;
        let buf = unsafe { core::slice::from_raw_parts(p3, len) };
        // transfer to usize
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

    #[test]
    fn table_register_test() {
        let mut table = Table::new();
        table.register(0, read);
        table.register(1, test);
        table.register(2, add);
        table.do_call(0, &[1, 2, 0, 0, 0, 0]);
        let data = [6usize; 8];
        table.do_call(1, &[0, 8 * 8, data.as_ptr() as usize]);
        let v = table.do_call(2, &[2, 4]);
        assert_eq!(v, Some(6));
    }
    #[test]
    fn register_macro_test() {
        let mut table = Table::new();
        register_syscall!(table, (0, read), (1, test), (2, add));
        let v = table.do_call(2, &[2, 4]);
        assert_eq!(v, Some(6));
    }
}
