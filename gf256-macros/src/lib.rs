
extern crate proc_macro;

mod common;
mod p;
mod gf;
#[cfg(feature="crc")] mod crc;


#[proc_macro_attribute]
pub fn p(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    p::p(args, input)
}

#[proc_macro_attribute]
pub fn gf(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    gf::gf(args, input)
}

#[cfg(feature="crc")]
#[proc_macro_attribute]
pub fn crc(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    crc::crc(args, input)
}
