use std::fmt::Debug;

use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Paren,
    Error as SynErr, Ident, Path, Token,
};

#[derive(Default, Debug)]
pub struct Args {
    pub name: Option<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // should not hard lock Arg to be path but effort
        let inner: Punctuated<Arg<Path>, Token!(,)> = Punctuated::parse_terminated(input)?;
        let mut new = Self::default();

        for arg in inner {
            match arg.name.to_string().as_str() {
                "name" => {
                    if let Some(ident) = arg.value.get_ident() {
                        if new.name.is_none() {
                            new.name = Some(ident.clone());
                        } else {
                            return Err(SynErr::new_spanned(arg, "name has already been set"));
                        }
                    } else {
                        let msg = format!(
                            "`{}` is not a valid struct identifier",
                            arg.value.to_token_stream()
                        );
                        return Err(SynErr::new_spanned(arg.value, msg));
                    }
                }
                e => {
                    return Err(SynErr::new_spanned(
                        arg.name,
                        format!("`{e}` is not a valid argument"),
                    ))
                }
            }
        }

        Ok(new)
    }
}

#[derive(Debug)]
struct Arg<T> {
    name: Ident,
    value: T,
    assignment_token: ArgAssignment,
}

impl<T: Parse> Parse for Arg<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let ident: Ident = input.parse()?;
            lookahead = input.lookahead1();

            if lookahead.peek(Token!(=)) {
                return Ok(Self {
                    name: ident,
                    assignment_token: ArgAssignment::Equals(input.parse()?),
                    value: input.parse()?,
                });
            } else if lookahead.peek(Paren) {
                let value;
                return Ok(Self {
                    name: ident,
                    assignment_token: ArgAssignment::Paren(parenthesized!(value in input)),
                    value: value.parse()?,
                });
            }
        }

        Err(lookahead.error())
    }
}

impl<T: ToTokens> ToTokens for Arg<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.assignment_token {
            ArgAssignment::Paren(paren) => {
                tokens.extend(quote_spanned!(paren.span.join()=> #self.name(#self.value)))
            }
            ArgAssignment::Equals(equals) => {
                tokens.extend(quote!(#self.name #equals #self.value));
            }
        }
    }
}

#[derive(Debug)]
enum ArgAssignment {
    Paren(Paren),
    Equals(Token!(=)),
}
