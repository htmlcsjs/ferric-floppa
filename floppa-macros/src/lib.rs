mod args;

use args::Args;
use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::Parser, parse_macro_input, parse_quote, Field, FnArg, Ident, ItemFn, Type};

/// Generates a proc macro from a function into an instance of MessageCommand TODO LINK
///
/// # Function formatting:
/// The function should be formatted like a normal rust function. The visibility of the generated
/// struct is inherated from the function, and aegs are provided via the following list
/// ## Function Args:
/// - TODO Fill in when added.
///
/// # Macro Args:
/// These are passed in the atturbute by either `arg(value)` or `arg = value`, a list of args
/// and what they do is provided below. These are all optional, and defaults are listed below
/// - `name` - The name of the struct, defaults to the function name in upper camel case
///
// this function is kinda fucked code wise, ignore it
#[proc_macro_attribute]
pub fn command(args: TokenStream, input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as ItemFn);

    let args = parse_macro_input!(args as Args);

    let soucrce_ident = &input.sig.ident;
    let name = if let Some(name) = args.name {
        name
    } else {
        format_ident!(
            "{}",
            soucrce_ident.to_string().to_upper_camel_case(),
            span = soucrce_ident.span()
        )
    };

    let visibility = &input.vis;
    let (impl_generics, ty_generics, where_clause) = input.sig.generics.split_for_impl();

    let mut types: Vec<&Type> = Vec::new();

    for i in &input.sig.inputs {
        let ty = get_arg_type(i);
        types.push(ty);
    }

    let data_name = format_ident!("{}GeneratedData", name);
    let data_type = decide_data_type(types, &data_name);
    let mod_name = format_ident!("__{}_internal", name);

    let expanded = quote! {
        #[allow(non_snake_case)]
        mod #mod_name {
            pub struct #name #ty_generics #where_clause;

            #data_type
            impl #impl_generics #name #ty_generics #where_clause {
                #![warn(non_snake_case)]
                #input
            }

            #[async_trait::async_trait]
            #[allow(unused_variables)]
            impl #impl_generics floppa::Command<'_> for #name #ty_generics #where_clause {
                type Data = #data_name;

                fn construct(cfg: &floppa::ThreadCfg, cli: &floppa::Cli, data: Self::Data) -> Self {
                    Self
                }

                async fn execute(&mut self,
                        event: &twilight_model::gateway::payload::incoming::MessageCreate,
                        http: std::sync::Arc<floppa::HttpClient>)
                    -> floppa::FlopResult<()>
                {
                    let msg = "todo";

                    http.create_message(event.channel_id)
                        .reply(event.id)
                        .content(&msg)?
                    .await?;
                    Ok(())
                }

                fn save(self) -> Self::Data {todo!()}

                fn raw(&self) -> &'static str {
                    "todo"
                }
            }
        }
        #visibility use #mod_name::#name;
    };

    TokenStream::from(expanded)
}

fn get_arg_type(arg: &FnArg) -> &Type {
    match arg {
        FnArg::Receiver(rec) => &rec.ty,
        FnArg::Typed(pat) => &pat.ty,
    }
}

fn decide_data_type(types: Vec<&Type>, name: &Ident) -> proc_macro2::TokenStream {
    if types.is_empty() {
        quote!(type #name = ();)
    } else if types.len() == 1 {
        let ty = types[0];
        quote!(type #name = #ty;)
    } else {
        let mut counter = 0;
        let fields = types.into_iter().map(|x| -> Field {
            counter += 1;
            Field::parse_named
                .parse_str(&format!("field{counter}:{}", x.to_token_stream()))
                .unwrap()
        });

        parse_quote!(
            #[derive(std::fmt::Debug, serde::Serialize, serde::Deserialize)]
            struct #name {
                #(#fields),*
            }
        )
    }
}
