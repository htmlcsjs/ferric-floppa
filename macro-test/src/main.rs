use floppa::Command;
use floppa_macros::command;

fn main() {
    dbg!();
}

#[command]
pub fn test(a: i16, b: i16) -> i32 {
    (a + b).into()
}
