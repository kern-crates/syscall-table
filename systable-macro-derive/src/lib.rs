#![cfg_attr(not(test), no_std)]

extern crate proc_macro;
extern crate alloc;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ parse_macro_input};


/// \[syscall_func(10)]
#[proc_macro_attribute]
pub fn syscall_func(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as syn::LitInt);
    let number = attr.base10_parse::<u16>().unwrap();
    let input = parse_macro_input!(item as syn::ItemFn);
    let ident = format_ident!("__syscall_{}", number);
    // println!("input = {:?}", input);
    let old_ident = input.sig.ident.clone();
    // let input_func_output = input.sig.output.clone();
    let name_ident = format_ident!("__{}" ,old_ident);
    // let init = format!(".init_array.{}", number);
    let name_syscall = quote!{
        #[inline]
        #[no_mangle]
         fn #name_ident(p:&[usize])->isize
             {
                let handler = register(#old_ident);
                let service = Service::from_handler(handler);
                service.handle(p)
            }
        submit!(
            ServiceWrapper{
                service:#name_ident,
                id:#number,
            }
        );
    };

    let expanded = quote! {
        #input
        mod #ident{
            use super::#old_ident;
            use syscall_table::{Service};
            use syscall_table::register;
            use syscall_table::{ServiceWrapper,submit};
            #name_syscall
        }
    };
    let stream = TokenStream::from(expanded);
    stream
}
