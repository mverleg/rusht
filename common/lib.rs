mod stdin;
mod err;

pub use self::stdin::stdin_lines;
pub use self::stdin::EmptyLineHandling;
pub use self::err::fail;
