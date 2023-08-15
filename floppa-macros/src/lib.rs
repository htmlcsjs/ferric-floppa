mod args;

use args::Args;
use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, Type};

/// Generates a proc macro from a function into an instance of MessageCommand TODO LINK
///
/// # Function formatting:
/// The function should be formatted like a normal rust function. The visibility of the generated
/// struct is inherated from the function, and aegs are provided via the following list
/// ## Function Args:
/// - TODO Fill in when added.
///
/// # Macro Args:
/// - `name` - The name of the struct, defaults to the function name in upper camel case
///
#[proc_macro_attribute]
pub fn command(args: TokenStream, input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as ItemFn);

    let args = parse_macro_input!(args as Args);

    let soucrce_ident = input.sig.ident;
    let name = if let Some(name) = args.name {
        name
    } else {
        format_ident!(
            "{}",
            soucrce_ident.to_string().to_upper_camel_case(),
            span = soucrce_ident.span()
        )
    };

    let visibility = input.vis;
    let (impl_generics, ty_generics, where_clause) = input.sig.generics.split_for_impl();
    let block = input.block;

    let expanded = quote! {
        #visibility struct #name #ty_generics #where_clause;

        #[async_trait::async_trait]
        impl #impl_generics floppa::Command<'_> for #name #ty_generics #where_clause {
            type Data = ();

            fn construct(_cfg: &floppa::ThreadCfg, _cli: &floppa::Cli, _data: Self::Data) -> Self {
                Self
            }

            async fn execute(&mut self,
                    event: &twilight_model::gateway::payload::incoming::MessageCreate,
                    http: std::sync::Arc<floppa::HttpClient>)
                -> floppa::FlopResult<()>
            {
                let msg = #block .to_string();

                http.create_message(event.channel_id)
                    .reply(event.id)
                    .content(&msg)?
                .await?;
                Ok(())
            }

            fn save(self) {}

            fn raw(&self) -> &'static str {
                ""
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_arg_type(arg: &FnArg) -> Box<Type> {
    match arg {
        FnArg::Receiver(rec) => rec.ty.clone(),
        FnArg::Typed(pat) => pat.ty.clone(),
    }
}
