use ::std::collections::HashMap;
use ::std::sync::RwLock;

use ::lazy_static::lazy_static;

lazy_static! {
    static ref TYPES: TypeRegistry = TypeRegistry::init();
}

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
        types.add_struct("Password");
        types.add_interface("Display");
        types
    }

    pub fn add_struct(&self, name: &str) -> Type {
        self.add_type(name, || TypeInfo {
            name: name.to_string(),
            kind: TypeKind::Struct {},
        })
    }

    pub fn add_interface(&self, name: &str) -> Type {
        self.add_type(name, || TypeInfo {
            name: name.to_string(),
            kind: TypeKind::Interface { sealed: false },
        })
    }

    fn add_type(&self, name: &str, info_gen: impl FnOnce() -> TypeInfo) -> Type {
        let mut content = self.content.write().expect("lock poisoned");
        let rank = content.all.len();
        if content.lookup.contains_key(name) {
            panic!("type already defined: {name}'")
        }
        content.all.push(info_gen());
        let typ = Type { id: rank };
        content.lookup.insert(name.to_owned(), typ);
        typ
    }

    pub fn lookup(&self, name: &str) -> Option<Type> {
        let content = self.content.read().expect("lock poisoned");
        content.lookup.get(name).map(|typ| *typ)
    }
}

#[derive(Debug)]
pub struct Constraint {}

#[derive(Debug)]
pub enum TypeKind {
    Struct {},
    Interface {
        sealed: bool,
    },
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
        let types = &TYPES.content.read().expect("lock poisoned").all;
        //TODO @mverleg: not ideal to access `content` directly, but whatevera
        let left = &types[self.id];
        let right = &types[value.id];
        match (&left.kind, &right.kind) {
            (TypeKind::Struct {}, _) => left.name == right.name,
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concrete_identical() {
        dbg!(&TYPES.content.read());
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
