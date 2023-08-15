use floppa::Command;
use floppa_macros::command;

fn main() {
    dbg!(TestCommand.raw());
}

#[command(name(TestCommand))]
pub fn test() -> i32 {
    4
}
