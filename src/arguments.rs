//  ARGUMENTS.rs
//    by Lut99
//
//  Description:
//!   Implements parsing arguments to our attribute macros.
//

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, ExprPath, Ident, Path, PathArguments, PathSegment, Token};


/***** LIBRARY *****/
/// Some abstraction of parsed arguments.
pub struct Arguments {
    /// The parsed arguments, as identifier to expression.
    pub args: Vec<(Option<Ident>, Expr)>,
}

// Parse
impl Parse for Arguments {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse key/value pairs... or just values!
        let mut args: Vec<(Option<Ident>, Expr)> = Vec::with_capacity(4);
        while !input.is_empty() {
            let mut key: Option<Ident> = Some(input.parse()?);
            let value: Expr = match input.parse::<Token![=]>() {
                Ok(_) => input.parse()?,
                Err(_) => Expr::Path(ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path:  Path {
                        leading_colon: None,
                        segments:      {
                            let mut segs = Punctuated::new();
                            segs.push(PathSegment { ident: key.take().unwrap(), arguments: PathArguments::None });
                            segs
                        },
                    },
                }),
            };
            args.push((key, value));

            // Pop optional punctuation
            let _ = input.parse::<Token![,]>();
        }
        Ok(Self { args })
    }
}
