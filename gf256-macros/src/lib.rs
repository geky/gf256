
extern crate proc_macro;

mod common;
mod p;
mod gf;
#[cfg(feature="lfsr")] mod lfsr;
#[cfg(feature="crc")] mod crc;
#[cfg(feature="shamir")] mod shamir;
#[cfg(feature="raid")] mod raid;
#[cfg(feature="rs")] mod rs;


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

#[cfg(feature="lfsr")]
#[proc_macro_attribute]
pub fn lfsr(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    lfsr::lfsr(args, input)
}

#[cfg(feature="crc")]
#[proc_macro_attribute]
pub fn crc(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    crc::crc(args, input)
}

#[cfg(feature="shamir")]
#[proc_macro_attribute]
pub fn shamir(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    shamir::shamir(args, input)
}

#[cfg(feature="raid")]
#[proc_macro_attribute]
pub fn raid(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    raid::raid(args, input)
}

#[cfg(feature="rs")]
#[proc_macro_attribute]
pub fn rs(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    rs::rs(args, input)
}
