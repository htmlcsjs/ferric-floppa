use std::process::Command;
fn main() {
    // Only rerun if the git head changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output();
    let git_hash = if let Ok(output) = output {
        String::from_utf8(output.stdout).unwrap_or_default()
    } else {
        String::new()
    };
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
