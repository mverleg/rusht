mod err;
mod stdin;

pub use self::err::fail;
pub use self::stdin::stdin_lines;
pub use self::stdin::EmptyLineHandling;
