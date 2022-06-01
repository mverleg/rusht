//TODO @mark: split into multiple mods

pub use self::dir_with::find_dir_with;
pub use self::dir_with::DirWithArgs;
pub use self::dir_with::Nested;
pub use self::dir_with::Order;
pub use self::dir_with::PathModification;
pub use self::unique::unique;
pub use self::unique::unique_prefix;
pub use self::unique::UniqueArgs;

mod dir_with;
mod unique;
