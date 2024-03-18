use tracing::error;

use crate::command::impls::*;

use super::ExtendedCommand;

const ERROR_MSG: &[u8] = "⚠️**ERROR**⚠️ Broken Command".as_bytes();

macro_rules! generate_array {
    ($first:tt, $($next:tt),+) => {
        1 + generate_array!($($next),+)
    };
    ($last:tt) => {
        1
    }
}

macro_rules! generate_construct {
    ($($cmd:ty),+) => {
        /// All the valid command names
        pub const VALID: [&str; generate_array!($($cmd),+)] = [$(stringify!($cmd)),+];
        /// Construct a command from a type, binary data and cli arguments
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
    AddCommand,
    InfoCommand,
    EditCommand,
    RemoveCommand,
    LinkCommand,
    RoleCommand,
    VersionCommand,
    FlopCountCommand,
    WikiCommand,
    StoikCommand
);
