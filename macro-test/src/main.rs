use floppa::Command;
use floppa_macros::command;

fn main() {
    dbg!(Test.raw());
}

#[command]
pub fn test() -> i32 {
    4
}
