use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::fmt;
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
    impls: HashSet<(Type, Type)>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let capacity = 128;
        TypeRegistry {
            content: RwLock::new(TypeRegistryContent {
                all: Vec::with_capacity(capacity),
                lookup: HashMap::with_capacity(capacity),
                impls: HashSet::with_capacity(capacity),
            })
        }
    }

    pub fn init() -> Self {
        let types = Self::new();
        let int = types.add_struct("int");
        let string = types.add_struct("string");
        let password = types.add_struct("Password");
        let display = types.add_interface("Display");
        types.implement(int, display);
        types.implement(string, display);
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

    pub fn add_sealed(&self, name: &str) -> Type {
        self.add_type(name, || TypeInfo {
            name: name.to_string(),
            kind: TypeKind::Interface { sealed: true },
        })
    }

    pub fn implement(&self, implementer: Type, abstraction: Type) {
        let mut content = self.content.write().expect("lock poisoned");
        let was_inserted = content.impls.insert((implementer, abstraction));
        assert!(was_inserted, "{implementer} already implements {abstraction}");
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

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cur = &TYPES.content.read().expect("lock poisoned").all[self.id];
        match cur.kind {
            TypeKind::Struct {} => write!(f, "struct ")?,
            TypeKind::Interface { sealed: true } => write!(f, "sealed ")?,
            TypeKind::Interface { sealed: false } => write!(f, "interface ")?,
        }
        write!(f, "{}", cur.name)
    }
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
