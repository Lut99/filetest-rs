//  ARGUMENTS.rs
//    by Lut99
//
//  Description:
//!   Implements parsing arguments to our attribute macros.
//

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, ExprLit, ExprPath, Ident, Lit, LitStr, Path, PathArguments, PathSegment, Token};


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
            let mut key: Option<Ident> = input.parse().ok();
            let value: Expr = match input.parse::<Token![=]>() {
                Ok(_) => {
                    if let Ok(s) = input.parse::<macro_string::MacroString>() {
                        Expr::Lit(ExprLit { attrs: Vec::new(), lit: Lit::Str(LitStr::new(&s.0, Span::call_site())) })
                    } else {
                        input.parse()?
                    }
                },
                Err(_) => match key.take() {
                    Some(key) => Expr::Path(ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path:  Path {
                            leading_colon: None,
                            segments:      {
                                let mut segs = Punctuated::new();
                                segs.push(PathSegment { ident: key, arguments: PathArguments::None });
                                segs
                            },
                        },
                    }),
                    None => input.parse()?,
                },
            };
            args.push((key, value));

            // Pop optional punctuation
            let _ = input.parse::<Token![,]>();
        }
        Ok(Self { args })
    }
}
