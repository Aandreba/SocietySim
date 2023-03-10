#![feature(iterator_try_collect, iter_intersperse)]

use std::{ffi::{CString}, ops::Deref, fs::File};
use derive_syn_parse::Parse;
use proc_macro2::{Span};
use quote::{quote, format_ident};
use syn::{parse_macro_input, punctuated::Punctuated, Token, LitStr, LitByteStr};

#[derive(Parse)]
struct Input {
    #[call(Punctuated::parse_terminated)]
    inputs: Punctuated<LitStr, Token![,]>
}

#[proc_macro]
pub fn cstr (str: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(str as LitStr);
    let str = input.value().into_bytes();
    
    match memchr::memchr(0, &str) {
        Some(nul_pos) => {
            return syn::Error::new_spanned(input, format!("Null byte found at index {nul_pos}")).into_compile_error().into();
        },
        None => {
            return quote! {{
                #[allow(unused_unsafe)]
                unsafe {
                    ::core::ffi::CStr::from_bytes_with_nul_unchecked(&[#(#str,)* 0u8])
                }
            }}.into()
        },
    };
}

#[proc_macro]
pub fn include_spv (path: proc_macro::TokenStream) -> proc_macro::TokenStream {
    macro_rules! tri {
        ($e:expr) => {
            match $e {
                Ok(x) => x,
                Err(e) => return syn::Error::new(Span::call_site(), e).into_compile_error().into()
            }
        };
    }

    let path = parse_macro_input!(path as LitStr).value();
    let path = tri!(std::env::var(&path));
    let mut file = tri!(File::open(&path));
    
    let spv = tri!(read_spv(&mut file));
    let len = spv.len();

    return quote! {{
        &[#(#spv),*] as &'static [u32; #len]
    }}.into()
}

#[proc_macro]
pub fn entry (item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Input { inputs } = parse_macro_input!(item as Input);

    let c_str = inputs.iter()
        .map(|x| CString::new(x.value()))
        .try_collect::<Vec<_>>();

    let c_str = match c_str {
        Ok(x) => x.into_iter().map(|x| LitByteStr::new(x.as_bytes_with_nul(), Span::call_site())),
        Err(e) => return syn::Error::new(Span::call_site(), e).into_compile_error().into()
    };

    let by_parts = inputs.iter()
        .map(|x| by_parts(x.value()))
        .collect::<Vec<_>>();

    let upper_case = by_parts.iter()
        .map(|x| x.iter()
            .map(Deref::deref)
            .map(str::to_uppercase)
            .intersperse("_".to_string())
            .collect::<String>()
        )
        .map(|x| format_ident!("{x}"))
        .collect::<Vec<_>>();

    let lower_case = by_parts.iter()
        .map(|x| x.iter()
            .map(Deref::deref)
            .map(str::to_lowercase)
            .intersperse("_".to_string())
            .collect::<String>()
        )
        .map(|x| format_ident!("{x}"))
        .collect::<Vec<_>>();

    let fn_types = by_parts.iter()
        .map(|x| format_ident!("Fn{}", x.join("")))
        .collect::<Vec<_>>();

    return quote! {
        #(
            const #upper_case: &CStr = unsafe {
                CStr::from_bytes_with_nul_unchecked(#c_str)
            };
        )*

        pub struct Entry {
            lib: Library,
            instance: NonZeroU64,
            #[cfg(unix)]
            pub(crate) get_instance_proc_addr: libloading::os::unix::Symbol<vk::FnGetInstanceProcAddr>,
            #[cfg(windows)]
            pub(crate) get_instance_proc_addr: libloading::os::windows::Symbol<vk::FnGetInstanceProcAddr>,
            pub(crate) create_instance: vk::FnCreateInstance,
            #(
                pub(crate) #lower_case: vk::#fn_types
            ),*
        }

        impl Entry {
            unsafe fn new (
                instance: NonZeroU64,
                lib: Library,
                create_instance: vk::FnCreateInstance,
                #[cfg(unix)] get_instance_proc_addr: libloading::os::unix::Symbol<vk::FnGetInstanceProcAddr>,
                #[cfg(windows)] get_instance_proc_addr: libloading::os::windows::Symbol<vk::FnGetInstanceProcAddr>,
            ) -> Self {
                return Self {
                    lib,
                    instance,
                    create_instance,
                    #(
                        #lower_case: transmute((get_instance_proc_addr)(instance.get(), #upper_case.as_ptr())),
                    )*
                    get_instance_proc_addr,
                }
            }
        }
    }.into()
}

#[inline]
fn by_parts (s: impl AsRef<str>) -> Vec<String> {
    let s = s.as_ref();
    assert!(s.starts_with("vk"));

    let mut current = s[2..3].to_string();
    let mut result = Vec::new();

    for c in s[3..].chars() {
        if c.is_uppercase() {
            result.push(core::mem::replace(&mut current, String::from(c)))
        } else {
            current.push(c)
        }
    }

    if !current.is_empty() { result.push(current) }
    return result;
}

#[inline]
fn read_spv (r: &mut File) -> std::io::Result<Vec<u32>> {
    const WORD_SIZE: usize = core::mem::size_of::<u32>();

    let mut words = Vec::<u32>::new();
    loop {
        let len = words.len();
        words.reserve(1);

        let bytes = unsafe {
            core::slice::from_raw_parts_mut(words.as_mut_ptr().add(len).cast::<u8>(), WORD_SIZE)
        };

        match std::io::Read::read(r, bytes)? {
            WORD_SIZE => unsafe { words.set_len(len + 1) },
            0 => break,
            _ => unreachable!()
        }
    }
    
    return Ok(words)
}