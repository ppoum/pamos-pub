use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

#[proc_macro_derive(Protocol)]
pub fn protocol_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let data = input.data;
    let data = match data {
        Data::Struct(d) => d,
        _ => panic!("Protocol can only be derived for structs"),
    };

    let fields = match data.fields {
        Fields::Unnamed(f) => f.unnamed,
        _ => panic!("Protocol can only be derived on tuple structs."),
    };

    if fields.len() != 1 {
        panic!("Expected one field, found {} instead", fields.len());
    }

    let field_type = &fields.first().unwrap().ty;

    quote! {
        impl crate::uefi::protocols::Protocol for #ident {
            fn try_locate(
                handle: crate::uefi::Handle,
                boot_services: &crate::uefi::BootServices
            ) -> Result<&Self, crate::uefi::protocols::ProtocolLocateError> {
                let raw = #field_type::try_locate_protocol(boot_services, handle)?;
                unsafe { Ok(&*(raw as *const Self)) }
            }
        }

    }
    .into()
}

#[proc_macro]
pub fn ucs2_slice(input: TokenStream) -> TokenStream {
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

    quote!([#(#chars),*]).into()
}

#[proc_macro]
pub fn guid_str_to_bytes(input: TokenStream) -> TokenStream {
    let str_input = parse_macro_input!(input as LitStr);
    let str = str_input.value();

    match try_guid_str_to_bytes(&str) {
        Ok(b) => quote!([#(#b),*]).into(),
        Err(e) => {
            let reason = match e {
                BadGuidString::BadFormat => {
                    "Wrong string format, expected hex string: aabbccdd-eeff-gghh-iijj-kkllmmnnoopp"
                }
                BadGuidString::InvalidCharacter => "Invalid character, expecting A-Fa-f0-9",
            };

            syn::Error::new(str_input.span(), reason)
                .to_compile_error()
                .into()
        }
    }
}

enum BadGuidString {
    BadFormat,
    InvalidCharacter,
}

/// String:  aabbccdd-eeff-gghh-iijj-kkllmmnnoopp
/// Becomes: ddccbbaaffeehhggiijjkkllmmnnoopp
/// Block #1: LE, #2: LE, #3: LE, #4: BE, #5: BE
fn try_guid_str_to_bytes(str: &str) -> Result<[u8; 16], BadGuidString> {
    // Check string format
    let blocks: Vec<&str> = str.split('-').collect();

    if blocks.len() != 5 {
        return Err(BadGuidString::BadFormat);
    }

    if blocks[0].len() != 8
        || blocks[1].len() != 4
        || blocks[2].len() != 4
        || blocks[3].len() != 4
        || blocks[4].len() != 12
    {
        return Err(BadGuidString::BadFormat);
    }

    let mut bytes = [0; 16];

    let x = hex_str_to_bytes(blocks[0], true).ok_or(BadGuidString::InvalidCharacter)?;
    bytes[..4].copy_from_slice(x.as_slice());
    let x = hex_str_to_bytes(blocks[1], true).ok_or(BadGuidString::InvalidCharacter)?;
    bytes[4..6].copy_from_slice(x.as_slice());
    let x = hex_str_to_bytes(blocks[2], true).ok_or(BadGuidString::InvalidCharacter)?;
    bytes[6..8].copy_from_slice(x.as_slice());
    let x = hex_str_to_bytes(blocks[3], false).ok_or(BadGuidString::InvalidCharacter)?;
    bytes[8..10].copy_from_slice(x.as_slice());
    let x = hex_str_to_bytes(blocks[4], false).ok_or(BadGuidString::InvalidCharacter)?;
    bytes[10..].copy_from_slice(x.as_slice());

    Ok(bytes)
}

fn hex_str_to_bytes(s: &str, little_endian: bool) -> Option<Vec<u8>> {
    let tmp = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok());
    if little_endian {
        tmp.rev().collect()
    } else {
        tmp.collect()
    }
}
