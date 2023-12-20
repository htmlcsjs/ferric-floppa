use tracing::error;

use crate::command::impls::*;

use super::ExtendedCommand;

const ERROR_MSG: &[u8] = "⚠️**ERROR**⚠️ Broken Command".as_bytes();

macro_rules! generate_construct {
    ($($cmd:ty),+) => {
        pub fn construct(
            ty: &str,
            data: &[u8],
            cli: &$crate::Cli,
        ) -> color_eyre::Result<Box<dyn ExtendedCommand + Send + Sync>> {
            Ok(match ty {
                $(
                    stringify!($cmd) => {
                        Box::new(<$cmd as ExtendedCommand>::construct(&cli, data)?) as Box<dyn ExtendedCommand + Send + Sync>
                    },
                )+
                _ => {
                    let msg = format!("{ty} is not a valid command type");
                    error!("{msg}");
                    Box::new(MessageCommand::construct(&cli, ERROR_MSG)?) as Box<dyn ExtendedCommand + Send + Sync>
                }
            })
        }
    };
}

// TODO: maybe allow for dynamially loaded plugins to register plugins
// also maybe name commands better
generate_construct!(
    MessageCommand,
    SubregistyMarkerCommand,
    RedirectMarkerCommand,
    AddCommand
);
