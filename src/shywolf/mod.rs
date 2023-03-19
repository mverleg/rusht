use ::std::collections::HashMap;
use ::std::sync::RwLock;

use ::lazy_static::lazy_static;

lazy_static! {
    static ref TYPES: TypeRegistry = TypeRegistry::init();
}

#[derive(Debug)]
pub struct TypeRegistry {
    content: RwLock<TypeRegistryContent>,
}

#[derive(Debug)]
pub struct TypeRegistryContent {
    all: Vec<TypeInfo>,
    lookup: HashMap<String, Type>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        TypeRegistry {
            content: RwLock::new(TypeRegistryContent {
                all: Vec::new(),
                lookup: HashMap::new(),
            })
        }
    }

    pub fn init() -> Self {
        let types = Self::new();
        types.add_struct("int");
        types.add_struct("string");
        types
    }

    pub fn add_struct(&self, name: &str) -> Type {
        let mut content = self.content.write().expect("lock poisoned");
        let rank = content.all.len();
        if content.lookup.contains_key(name) {
            panic!("type already defined: {name}'")
        }
        content.all.push(TypeInfo {
            name: name.to_string(),
            kind: TypeKind::Struct {},
        });
        Type { id: rank }
    }

    pub fn lookup(&self, name: &str) -> Option<Type> {
        let content = self.content.read().expect("lock poisoned");
        content.lookup.get(name).map(|typ| *typ)
    }

    pub fn with<'a, R>(&self, op: &'a impl FnOnce(dyn Fn(Type) -> &'a TypeInfo) -> R) -> R {
        let content = self.content.read().expect("lock poisoned");
        op(|typ: Type| content.all.get(typ.id).expect("non-existent Type instance"))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Type {
    id: usize,
}

impl Type {
    /// is this valid?
    /// x: ThisType = ArgumentType::new()
    pub fn is_assignable_from(&self, value: Type) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concrete_identical() {
        let nr = TYPES.lookup("int").unwrap();
        assert!(nr.is_assignable_from(nr));
    }

    #[test]
    fn test_concrete_mismatch_structs() {
        let nr = TYPES.lookup("int").unwrap();
        let text = TYPES.lookup("string").unwrap();
        assert!(!nr.is_assignable_from(text));
    }
}
