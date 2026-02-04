//  FILE A/B TEST.rs
//    by Lut99
//
//  Description:
//!   Implements the [`file_ab_test()`](super::file_ab_test())-macro.
//

use std::borrow::Cow;
use std::path::PathBuf;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens as _, quote};
use syn::spanned::Spanned as _;
use syn::{Expr, ExprLit, Ident, ItemFn, Lit};

use crate::arguments::Arguments;


/***** ARGUMENTS *****/
/// Specific arguments for this macro.
struct FileABTestArgs {
    /// The suffix of input files.
    input_suffix: String,
    /// The suffix of gold files.
    gold_suffix: String,
    /// The path where to look.
    path: PathBuf,
    /// Some span to point to with path errors.
    path_span: Span,
}
impl TryFrom<Arguments> for FileABTestArgs {
    type Error = syn::Error;

    #[inline]
    fn try_from(value: Arguments) -> Result<Self, Self::Error> {
        fn key_is(key: &Option<Ident>, name: &str) -> bool {
            let skey = key.as_ref().map(Ident::to_string);
            if let Some(k) = skey.as_ref().map(String::as_str) { k == name } else { false }
        }

        // Either position 0, or the `input`-key.
        let mut pos_i: usize = 0;
        let mut input_suffix = None;
        let mut gold_suffix = None;
        let mut path = None;
        for (key, value) in value.args {
            if key_is(&key, "input") || (key.is_none() && pos_i == 0) {
                let Expr::Lit(ExprLit { lit: Lit::Str(suffix), .. }) = value else {
                    return Err(syn::Error::new(value.span(), "Expected a string literal value for 'input'"));
                };
                input_suffix = Some(suffix.value());
                if key.is_none() {
                    pos_i += 1
                }
            } else if key_is(&key, "gold") || (key.is_none() && pos_i == 1) {
                let Expr::Lit(ExprLit { lit: Lit::Str(suffix), .. }) = value else {
                    return Err(syn::Error::new(value.span(), "Expected a string literal value for 'gold'"));
                };
                gold_suffix = Some(suffix.value());
                if key.is_none() {
                    pos_i += 1
                }
            } else if key_is(&key, "path") || (key.is_none() && pos_i == 2) {
                let Expr::Lit(ExprLit { lit: Lit::Str(suffix), .. }) = value else {
                    return Err(syn::Error::new(value.span(), "Expected a string literal value for 'path'"));
                };
                path = Some((suffix.value().into(), suffix.span()));
                if key.is_none() {
                    pos_i += 1
                }
            } else if let Some(key) = key {
                return Err(syn::Error::new(key.span(), "Unknown argument {key}"));
            } else {
                return Err(syn::Error::new(value.span(), "Unknown argument at position {pos_i}"));
            }
        }

        // OK, now require all of the arguments to be given
        let input_suffix = input_suffix.ok_or_else(|| syn::Error::new(Span::call_site(), "Missing argument 'input' (position 0)"))?;
        let gold_suffix = gold_suffix.ok_or_else(|| syn::Error::new(Span::call_site(), "Missing argument 'gold' (position 1)"))?;
        let (path, path_span) = path.ok_or_else(|| syn::Error::new(Span::call_site(), "Missing argument 'path' (position 2)"))?;
        Ok(Self { input_suffix, gold_suffix, path, path_span })
    }
}





/***** LIBRARY *****/
/// Actually implements the macro.
///
/// # Arguments
/// - `attrs`: A [`TokenStream2`] encoding what is given as attributes (if any).
/// - `input`: The contents to transform.
///
/// # Returns
/// A [`TokenStream2`] encoding what to do.
///
/// # Errors
/// This function may error if we failed to parse the input correctly.
pub fn handle(attrs: TokenStream2, input: TokenStream2) -> Result<TokenStream2, syn::Error> {
    let args: Arguments = syn::parse2(attrs)?;
    let FileABTestArgs { input_suffix, gold_suffix, path, path_span } = args.try_into()?;

    // Parse the input
    let input: ItemFn = syn::parse2(input)?;

    // Find the files in the path (best-effort)
    let path: PathBuf = if path.is_absolute() {
        path
    } else {
        let base = PathBuf::from(proc_macro::Span::call_site().file());
        if let Some(parent) = base.parent() { parent.join(path) } else { base.join(path) }
    };

    // Check what's what with the resulting path
    let mut files: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut todo: Vec<PathBuf> = vec![path];
    while let Some(next) = todo.pop() {
        if next.is_file() {
            // Check if it ends with the input suffix
            let snext: Cow<str> = next.as_os_str().to_string_lossy();
            let (base, ext): (&str, &str) = match snext.find('.') {
                Some(pos) => (&snext[..pos], &snext[pos..]),
                None => (snext.as_ref(), ""),
            };
            if ext != input_suffix {
                continue;
            }

            // Attempt to find a gold file
            let gold: PathBuf = format!("{base}{gold_suffix}").into();
            if !gold.is_file() {
                return Err(syn::Error::new(path_span, &format!("Gold file {gold:?} for input file {next:?} not found or not a file")));
            }
            files.push((next, gold));
        } else if next.is_dir() {
            // Search the directory (recursively) for all files
            for (i, entry) in
                std::fs::read_dir(&next).map_err(|err| syn::Error::new(path_span, &format!("Failed to read directory {next:?} ({err})")))?.enumerate()
            {
                todo.push(
                    entry.map_err(|err| syn::Error::new(path_span, &format!("Failed to read {i}th entry in directory {next:?} ({err})")))?.path(),
                );
            }
        } else {
            return Err(syn::Error::new(path_span, &format!("Path {next:?} is neither a file nor a directory")));
        };
    }

    // Now generate the code for every test we found
    let mut res = input.to_token_stream();
    for (test, gold) in files {
        let stest: Cow<str> = test.as_os_str().to_string_lossy();
        let sgold: Cow<str> = gold.as_os_str().to_string_lossy();

        // Preprocess the path to get a unique name
        let input_name: &Ident = &input.sig.ident;
        let mut name: String = input_name.to_string();
        name.push('_');
        for c in stest.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                name.push(c);
            } else {
                name.push('_');
            }
        }
        let name = Ident::new(&name, input_name.span());

        // Write it with the test macro prefix
        res.extend(quote! {
            #[automatically_derived]
            #[test]
            fn #name() {
                // Load the inputs
                let input_path: ::std::path::PathBuf = ::std::path::PathBuf::from(#stest);
                let gold_path: ::std::path::PathBuf = ::std::path::PathBuf::from(#sgold);
                let input: ::std::vec::Vec<::std::primitive::u8> = <::std::result::Result<_, _>>::unwrap_or_else(::std::fs::read(&input_path), |err| ::std::panic!("Failed to read input file {input_path:?}: {err}"));
                let gold: ::std::vec::Vec<::std::primitive::u8> = <::std::result::Result<_, _>>::unwrap_or_else(::std::fs::read(&gold_path), |err| ::std::panic!("Failed to read input file {gold_path:?}: {err}"));

                // Run the given function
                #input_name(input_path, input, gold_path, gold);
            }
        });
    }

    // Done
    Ok(res)
}
