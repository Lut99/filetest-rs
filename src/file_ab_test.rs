//  FILE A/B TEST.rs
//    by Lut99
//
//  Description:
//!   Implements the [`file_ab_test()`](super::file_ab_test())-macro.
//

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens as _, quote};
use syn::spanned::Spanned as _;
use syn::{Expr, ExprLit, Ident, ItemFn, Lit};

use crate::arguments::Arguments;


/***** HELPER FUNCTIONS *****/
/// Turns a given string into something safe for a function name.
///
/// # Arguments
/// - `raw`: Some raw text to make safe.
///
/// # Returns
/// A safe version of the raw text. Might just be the input [`str`] if it was safe to begin with.
fn safeify(raw: &str) -> Cow<'_, str> {
    let mut safe: Option<String> = None;
    for (i, c) in raw.char_indices() {
        if c.is_alphanumeric() || c == '_' {
            // Only push if there is a buffer to write to, else we're just confirming this is OK
            if let Some(safe) = &mut safe {
                safe.push(c);
            }
        } else {
            // Ensure the buffer is initialized and then add the replacement char
            if safe.is_none() {
                safe = Some(raw[..i].into());
            }
            safe.as_mut().unwrap().push('_');
        }
    }
    match safe {
        Some(safe) => Cow::Owned(safe),
        None => Cow::Borrowed(raw),
    }
}





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
    let mut files: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut todo: Vec<PathBuf> = vec![path];
    while let Some(next) = todo.pop() {
        if next.is_file() {
            // Canonicalize the filename first
            let next: PathBuf = next.canonicalize().unwrap_or_else(|err| panic!("Failed to canonicalize input file path {next:?}: {err}"));
            let snext: Cow<str> = next.to_string_lossy();

            // Check if it ends with the input suffix
            let (base, ext): (&str, &str) = match snext.find('.') {
                Some(pos) => (&snext[..pos], &snext[pos..]),
                None => (snext.as_ref(), ""),
            };
            if ext != input_suffix {
                continue;
            }

            // Attempt to find the gold file
            let gold: PathBuf = format!("{base}{gold_suffix}").into();
            if !gold.is_file() {
                return Err(syn::Error::new(path_span, &format!("Gold file {gold:?} for input file {next:?} not found or not a file")));
            }

            // Store the file as the pair
            files.insert(next, gold);
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

    // Generate useful names for all tests
    let mut names: HashMap<String, Vec<&Path>> = HashMap::with_capacity(files.len());
    for input_path in files.keys() {
        // Get the base name
        let sinput_file: Cow<str> = input_path.file_name().unwrap_or(input_path.as_os_str()).to_string_lossy();
        let sbase: &str = sinput_file.find('.').map(|pos| &sinput_file[..pos]).unwrap_or(sinput_file.as_ref());
        let name: Cow<str> = safeify(sbase);

        // For now, just store it and collect the paths
        names.entry(name.into()).or_default().push(input_path.as_path());
    }

    // If there is a non-unique name, then attempt to resolve it by adding paths to it
    let input_name: &Ident = &input.sig.ident;
    let mut final_names: HashMap<String, &Path> = HashMap::with_capacity(files.len());
    for (name, mut paths) in names.into_iter() {
        // If it's a unique name, we're done
        if paths.len() == 1 {
            final_names.insert(format!("{input_name}_{name}"), paths[0]);
            continue;
        }
        paths.sort();

        // If there's multiple, then see if there are unique paths (excl. the filename) among them
        let parents: Vec<&Path> = paths.iter().copied().map(Path::parent).map(|p| p.unwrap_or("".as_ref())).collect();
        'p1: for (i, p1) in parents.iter().copied().enumerate() {
            let mut components: Vec<Component> = p1.components().collect();
            for (j, p2) in parents.iter().copied().enumerate() {
                // Don't compare itself
                if i == j {
                    continue;
                }

                if p1 == p2 {
                    // The path is not unique! We must instead create a unique name using numeric suffixes.
                    for j in 1.. {
                        let attempt = format!("{input_name}_{name}{j}");
                        if !final_names.contains_key(&attempt) {
                            final_names.insert(attempt, paths[i]);
                            continue 'p1;
                        }
                    }
                }

                // It's not unique so far! While we have it, strip the shared prefix and keep the
                // shortest "unique" path we have
                let unique_components: Vec<Component> = p1
                    .components()
                    .map(Some)
                    .zip(p2.components().map(Some).chain(std::iter::repeat(None)))
                    .skip_while(|(c1, c2)| c1 == c2)
                    .map(|(c1, _)| c1.unwrap())
                    .collect();
                if unique_components.len() < components.len() {
                    components = unique_components;
                }
            }

            // `p1` is a unique path! We can store it using the unique components
            final_names.insert(
                format!(
                    "{}_{}{}",
                    input_name,
                    components.into_iter().map(|c| format!("{}_", safeify(c.as_os_str().to_string_lossy().as_ref()))).collect::<String>(),
                    name
                ),
                paths[i],
            );
        }
    }

    // Now generate the code for every test we found
    let mut res = input.to_token_stream();
    for (name, input_path) in final_names {
        let gold_path: &Path = files.get(input_path).unwrap();
        let sinput_path: Cow<str> = input_path.to_string_lossy();
        let sgold_path: Cow<str> = gold_path.to_string_lossy();

        // Write it with the test macro prefix
        let iname = Ident::new(&name, input_name.span());
        res.extend(quote! {
            #[automatically_derived]
            #[allow(non_snake_case)]
            #[test]
            fn #iname() {
                // Load the inputs
                let input_path: ::std::path::PathBuf = ::std::path::PathBuf::from(#sinput_path);
                let gold_path: ::std::path::PathBuf = ::std::path::PathBuf::from(#sgold_path);
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
