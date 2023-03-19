use ::std::collections::HashSet;
use ::std::sync::RwLock;

use ::lazy_static::lazy_static;

lazy_static! {
    static ref TYPES: RwLock<TypeRegistry> = RwLock::new(TypeRegistry::new());
}

#[derive(Debug)]
pub struct TypeRegistry {
    all: Vec<TypeInfo>,
    seen: HashSet<String>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        TypeRegistry {
            all: Vec::new(),
            seen: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct Constraint {}

#[derive(Debug)]
pub enum TypeKind {
    Struct {},
    Interface {},
    Sealed {},
}

#[derive(Debug)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
}

#[derive(Debug)]
pub struct Type {}

pub fn define_struct(name: &str) -> Type {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concrete_match() {
        let int = define_struct("int");
        let real = define_struct("f64");
    }

    #[test]
    fn test_concrete_mismatch() {
        todo!()
    }
}
