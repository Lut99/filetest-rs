//  LIB.rs
//    by Lut99
//
//  Description:
//!   Procedural macro library for generating unit tests based on files.
//!   
//!   
//!   # Installation
//!   To use this macro in one of your projects, add the following to your `Cargo.toml` file:
//!   ```toml
//!   [dependencies]
//!   filetest = { git = "https://github.com/Lut99/filetest-rs" }
//!   ```
//!   Optionally, you can commit to a specific tag:
//!   ```toml
//!   [dependencies]
//!   filetest = { git = "https://github.com/Lut99/filetest-rs", tag = "v0.1.0" }
//!   ```
//!   
//!   
//!   # Usage
//!   This library contributes one procedural macro.
//!   
//!   ## `#[file_ab_test]`
//!   Adding this attribute to your test will create a unit test for every file with a specific suffix.
//!   Then, for each of those files, a "gold" file with the correct answer is looked for. It then calls
//!   your test function for each of these pairs.
//!   
//!   See the documentation of the macro for more information.
//!   
//!   
//!   # Features
//!   This crate does currently not support any features.
//!   
//!   
//!   # Contributions
//!   Contributions to this crate are welcome! Please
//!   [open an issue](https://github.com/Lut99/filetest-rs/issues) or, if you are particularly
//!   industrious, a [pull request](https://github.com/Lut99/filetest-rs/pulls).
//!   
//!   
//!   # License
//!   This code is licensed under the Apache 2.0 license. See [`LICENSE`](./LICENSE) for more details.
//

// Modules
mod arguments;
mod file_ab_test;

// Imports
use proc_macro::TokenStream;


/***** LIBRARY *****/
#[doc = include_str!("../docs/file_ab_test.md")]
#[proc_macro_attribute]
pub fn file_ab_test(attrs: TokenStream, input: TokenStream) -> TokenStream {
    match file_ab_test::handle(attrs.into(), input.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
