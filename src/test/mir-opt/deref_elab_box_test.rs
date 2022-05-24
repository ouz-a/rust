#[cfg(feature = "std")]
use std::error;
use std::io;
use std::io::{Error as OtherError, ErrorKind};

pub struct Error {
    err: Box<ErrorImpl>,
}
struct ErrorImpl {
    code: ErrorCode,
    line: usize,
    column: usize,
}
pub enum ErrorCode {
    Message(Box<str>),
    Io(io::Error),
}
impl Error {
    pub fn fix_position<F>(self, f: F) -> Self
    where
        F: FnOnce(ErrorCode) -> Error,
    {
        if self.err.line == 0 {
            f(self.err.code)
        } else {
            self
        }
    }
    pub fn io(error: io::Error) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::Io(error),
                line: 0,
                column: 0,
            }),
        }
    }
}
pub fn ret_er(_er: ErrorCode) -> Error {
    let custom_error = OtherError::new(ErrorKind::Other, "oh no!");
    let err_impl = ErrorImpl {
        code: ErrorCode::Io(custom_error),
        line: 0,
        column: 0,
    };

    Error {
        err: Box::new(err_impl),
    }
}

fn main() {
    let custom_error = OtherError::new(ErrorKind::Other, "oh no!");
    let err_code = ErrorCode::Io(custom_error);
    let err = ret_er(err_code);
    err.fix_position(ret_er);
}
// EMIT_MIR deref_elab_box_test.main.Derefer.diff
