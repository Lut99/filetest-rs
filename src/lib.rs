//  LIB.rs
//    by Lut99
//
//  Description:
//!   TODO
//

// Modules
mod arguments;
mod file_ab_test;

// Imports
use proc_macro::TokenStream;


/***** LIBRARY *****/
#[doc = concat!("Proc macro for doing A/B testing on pairs of files.\n\nThis macro will take your function and create tests for every file in the given directory.\n\n# Examples\n```ignore", include_str!("../tests/file_ab_test.rs"), "```\n")]
#[proc_macro_attribute]
pub fn file_ab_test(attrs: TokenStream, input: TokenStream) -> TokenStream {
    match file_ab_test::handle(attrs.into(), input.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
