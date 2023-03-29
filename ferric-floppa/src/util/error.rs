use tracing::error;

use crate::consts::FlopError;

#[inline]
pub fn report_error(err: Box<impl FlopError + ?Sized>, msg: &str) {
    let mut msg = format!("{msg}: {err}");
    let mut src = err.source();
    while let Some(src_e) = src {
        msg.push_str(&format!("; Caused by: {src_e:?}"));
        src = src_e.source();
    }
    error!(msg);
    dbg!()
}
