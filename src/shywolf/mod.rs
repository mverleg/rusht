use ::std::collections::HashSet;
use ::std::sync::RwLock;

use ::lazy_static::lazy_static;

lazy_static! {
    static ref TYPES: TypeRegistry = TypeRegistry::new();
}

#[derive(Debug)]
pub struct TypeRegistry {
    content: RwLock<TypeRegistryContent>,
}

#[derive(Debug)]
pub struct TypeRegistryContent {
    all: Vec<TypeInfo>,
    seen: HashSet<String>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        TypeRegistry {
            content: RwLock::new(TypeRegistryContent {
                all: Vec::new(),
                seen: HashSet::new(),
            })
        }
    }

    pub fn add_struct(&self, name: &str) -> Type {
        let mut content = self.content.write().expect("lock poisoned");
        assert!(content.seen.insert(name.to_owned()));
        let rank = content.all.len();
        content.all.push(TypeInfo {
            name: name.to_string(),
            kind: TypeKind::Struct {},
        });
        Type { id: rank }
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

#[derive(Debug, Clone, Copy)]
pub struct Type {
    id: usize,
}

impl Type {
    /// is this valid?
    /// x: ThisType = ArgumentType::new()
    pub fn is_assignable_from(&self, value: Type) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concrete_identical() {
        let nr = TYPES.add_struct("int");
        assert!(nr.is_assignable_from(nr));
    }

    #[test]
    fn test_concrete_mismatch_structs() {
        let nr = TYPES.add_struct("int");
        let txt = TYPES.add_struct("string");
        assert!(!nr.is_assignable_from(txt));
    }
}
