use tracing::error;

use crate::command::MessageCommand;

use super::Command;

macro_rules! generate_construct {
    ($($cmd:ty),+) => {
        pub fn construct(
            ty: &str,
            data: rmpv::Value,
            cli: &$crate::Cli,
        ) -> Box<dyn Command + Send + Sync> {
            match ty {
                $(
                    stringify!($cmd) => {
                        Box::new(<$cmd>::construct(&cli, data))
                    },
                )+
                _ => {
                    let msg = format!("{ty} is not a valid command type");
                    error!("{msg}");
                    Box::new(MessageCommand::construct(&cli, data))
                }
            }
        }
    };
}

// TODO: maybe allow for dynamially loaded plugins to register plugins
// also maybe name commands better
generate_construct!(MessageCommand);
