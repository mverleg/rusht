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

#[derive(Debug)]
pub enum DefineErr {
    AlreadyExists(String),
}

#[derive(Debug)]
pub enum ImplErr {
    Circular(Type, Type),
    CannotImplStruct(Type),
    AlreadyImpl(Type, Type),
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
        let int = types.add_struct("int").unwrap();
        let double = types.add_struct("double").unwrap();
        let string = types.add_struct("string").unwrap();
        types.add_struct("Password");
        let display = types.add_interface("Display").unwrap();
        types.implement(int, display).unwrap();
        types.implement(double, display).unwrap();
        types.implement(string, display).unwrap();
        let number = types.add_interface("Number").unwrap();
        let add = types.add_interface("Add").unwrap();
        let sub = types.add_interface("Sub").unwrap();
        let mul = types.add_interface("Mul").unwrap();
        let div = types.add_interface("Div").unwrap();
        types.implement(number, add).unwrap();
        types.implement(number, sub).unwrap();
        types.implement(number, mul).unwrap();
        types.implement(number, div).unwrap();
        //TODO @mverleg: does order matter? i.e. if int impl number, and then number impl add, does int still require number?
        //TODO @mverleg: I think we must collect all impls, and then at the end test that transitive impls are satisfied
        types.implement(int, number).unwrap();
        types.implement(double, number).unwrap();
        types
    }

    pub fn add_struct(&self, name: &str) -> Result<Type, DefineErr> {
        self.add_type(name, || TypeInfo {
            name: name.to_string(),
            kind: TypeKind::Struct {},
        })
    }

    pub fn add_interface(&self, name: &str) -> Result<Type, DefineErr> {
        self.add_type(name, || TypeInfo {
            name: name.to_string(),
            kind: TypeKind::Interface { sealed: false },
        })
    }

    pub fn add_sealed(&self, name: &str) -> Result<Type, DefineErr> {
        self.add_type(name, || TypeInfo {
            name: name.to_string(),
            kind: TypeKind::Interface { sealed: true },
        })
    }

    fn add_type(&self, name: &str, info_gen: impl FnOnce() -> TypeInfo) -> Result<Type, DefineErr> {
        let mut content = self.content.write().expect("lock poisoned");
        let rank = content.all.len();
        if content.lookup.contains_key(name) {
            return Err(DefineErr::AlreadyExists(name.to_owned()))
        }
        content.all.push(info_gen());
        let typ = Type { id: rank };
        content.lookup.insert(name.to_owned(), typ);
        Ok(typ)
    }

    pub fn implement(&self, implementer: Type, abstraction: Type) -> Result<(), ImplErr> {
        let mut content = self.content.write().expect("lock poisoned");
        if let TypeKind::Struct { .. } = content.all[abstraction.id].kind {
            return Err(ImplErr::CannotImplStruct(abstraction))
        }
        if content.impls.contains(&(abstraction, implementer)) {
            return Err(ImplErr::Circular(implementer, abstraction))
        }
        if content.impls.contains(&(implementer, abstraction)) {
            return Err(ImplErr::AlreadyImpl(implementer, abstraction))
        }
        assert!(content.impls.insert((implementer, abstraction)));
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<Type> {
        let content = self.content.read().expect("lock poisoned");
        content.lookup.get(name).map(|typ| *typ)
    }
}

#[derive(Debug)]
pub struct Constraint {
    and_bounds: HashSet<Type>,
    //TODO @mverleg: ordered set
}

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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self, self.id)
    }
}

impl Type {
    /// is this valid?
    /// x: ThisType = ArgumentType::new()
    pub fn is_assignable_from(&self, value: Type) -> bool {
        let types = &TYPES.content.read().expect("lock poisoned");
        //TODO @mverleg: not ideal to access `content` directly, but whatevera
        let left = &types.all[self.id];
        let right = &types.all[value.id];
        match (&left.kind, &right.kind) {
            (TypeKind::Struct {}, _) => {
                left.name == right.name
            },
            (TypeKind::Interface { sealed: _ }, TypeKind::Struct {}) => {
                types.impls.contains(&(value, *self))
                //TODO @mverleg: ^ this is only valid as long as interfaces cannot extend/impl other interfaces
            },
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_assign_struct_to_same_struct() {
        let nr = TYPES.lookup("int").unwrap();
        assert!(nr.is_assignable_from(nr));
    }

    #[test]
    fn cannot_assign_struct_to_different_struct() {
        let nr = TYPES.lookup("int").unwrap();
        let text = TYPES.lookup("string").unwrap();
        assert!(!nr.is_assignable_from(text));
    }

    #[test]
    fn can_assign_interface_to_interface() {
        let display = TYPES.lookup("Display").unwrap();
        let nr = TYPES.lookup("int").unwrap();
        assert!(!nr.is_assignable_from(display));
    }

    #[test]
    fn cannot_assign_interface_to_struct() {
        let display = TYPES.lookup("Display").unwrap();
        let nr = TYPES.lookup("int").unwrap();
        assert!(!nr.is_assignable_from(display));
    }

    #[test]
    fn can_assign_struct_to_interface_if_impl() {
        let display = TYPES.lookup("Display").unwrap();
        let nr = TYPES.lookup("int").unwrap();
        assert!(display.is_assignable_from(nr));
    }

    #[test]
    fn cannot_assign_struct_to_interface_if_not_impl() {
        let display = TYPES.lookup("Display").unwrap();
        let password = TYPES.lookup("Password").unwrap();
        assert!(!display.is_assignable_from(password));
    }
}
