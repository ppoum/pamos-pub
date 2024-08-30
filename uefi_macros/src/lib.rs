use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn cstr16(input: TokenStream) -> TokenStream {
    let str = parse_macro_input!(input as LitStr).value();

    // Convert string to utf16 array
    let mut chars: Vec<u16> = str
        .chars()
        .map(|c| {
            if c as u32 > 0xFFFF {
                panic!("Invalid character: {}", c);
            }
            c as u16
        })
        .collect();
    chars.push(0); // NULL terminator

    quote!(unsafe { ::lib::uefi::CStr16::from_u16_unsafe(&[#(#chars),*]) }).into()
}
