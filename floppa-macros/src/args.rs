use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, ExprCall, Ident, Token,
};

#[derive(Default, Debug)]
pub struct Args {
    pub name: Option<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // having this be a call expression is not the greatest but icba to write my own item
        let inner: Punctuated<ExprCall, Token!(,)> = Punctuated::parse_terminated(input)?;
        let mut new = Self::default();

        for call in inner {
            if let Expr::Path(expr) = *call.func {
                if expr.path == syn::parse_str("name")? {
                    let args = call.args.iter().collect::<Vec<_>>();
                    if args.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            call.args,
                            "This value expects one argument",
                        ));
                    } else if new.name.is_some() {
                        return Err(syn::Error::new_spanned(
                            call.args,
                            "Name has already been set",
                        ));
                    } else {
                        if let Expr::Path(path) = args[0] {
                            new.name = Some(
                                path.path
                                    .get_ident()
                                    .ok_or(syn::Error::new_spanned(
                                        path,
                                        "This is not a valid identifier",
                                    ))?
                                    .clone(),
                            );
                        } else {
                            let msg =
                                format!("`{}` is not an identifier", args[0].to_token_stream());
                            return Err(syn::Error::new_spanned(args[0], msg));
                        }
                    }
                } else {
                    let msg = format!("`{}` is not a supported value", expr.to_token_stream());
                    return Err(syn::Error::new_spanned(expr, msg));
                }
            } else {
                let msg = format!(
                    "`{}` is not a recognised value",
                    call.func.to_token_stream()
                );
                return Err(syn::Error::new_spanned(call, msg));
            }
        }

        Ok(new)
    }
}
