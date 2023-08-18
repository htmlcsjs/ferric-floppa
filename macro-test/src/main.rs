use floppa::Command;
use floppa_macros::command;

fn main() {
    dbg!(TestCommand.raw());
}

#[command(name(TestCommand))]
pub fn test(a: i16) -> i32 {
    a.into()
}
