
//TODO @mverleg: scopes

use ::std::collections::HashMap;
use ::std::rc::Rc;
use ::std::sync::atomic::AtomicU32;
use ::std::sync::atomic::Ordering;
use std::collections::hash_map::Entry;
use std::hash;
use std::hash::Hasher;

static DUMMY_LOC_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Source file location; dummy for now
#[derive(Debug, Clone, PartialEq)]
struct Loc {
    pub pos: u32,
}

#[cfg(test)]
impl Loc {
    pub fn dummy() -> Loc {
        Loc { pos: DUMMY_LOC_COUNTER.fetch_add(1, Ordering::AcqRel) }
    }
}

#[allow(unused)]  // used later in error handling
#[derive(Debug)]
enum TypeErr {
    DoubleDeclaration { existing: Type, duplicate_kind: TypeKind, duplicate_loc: Loc },
    /// implement an abstraction for a type that doesn't exist; shouldn't really be possible with current syntax plan
    NonExistentImplementer { implementer: Identifier, abstraction: Identifier, impl_loc: Loc },
    NonExistentAbstraction { implementer: Type, abstraction: Identifier, impl_loc: Loc },
    DuplicateImplementation { implementer: Type, abstraction: Type, first_loc: Loc, duplicate_loc: Loc },
    StructAbstraction { implementer: Type, abstraction_struct: Type, impl_loc: Loc },
    ImplementationCycle { cycle: Vec<Type>, impl_loc: Loc },
}

#[derive(Debug, Clone)]
struct Type {
    id: usize,
    info: Rc<TypeInfo>,
}

impl Type {
    pub fn of(info: Rc<TypeInfo>) -> Self {
        Type {
            id: info.id,
            info,
        }
    }

    pub fn name(&self) -> &str {
        &self.info.name.text
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Type {}

impl hash::Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.id)
    }
}

#[derive(Debug, PartialEq)]
enum TypeKind {
    Struct,
    Interface { sealed: bool, },
}

#[derive(Debug)]
struct TypeInfo {
    id: usize,
    name: Identifier,
    kind: TypeKind,
    declaration_loc: Loc,
}

impl TypeInfo {
    pub fn typ(self: &Rc<Self>) -> Type {
        Type::of(self.clone())
    }
}

#[derive(Debug)]
struct AST {
    structs: Vec<(Identifier, Loc)>,
    interfaces: Vec<(Identifier, Loc, bool)>,
    implementations: Vec<(Identifier, Identifier, Loc)>,
}

impl AST {
    pub fn new() -> Self {
        AST {
            structs: Vec::new(),
            interfaces: Vec::new(),
            implementations: Vec::new(),
        }
    }

    pub fn declare_struct(&mut self, name: impl Into<Identifier>, loc: Loc) {
        self.structs.push((name.into(), loc));
    }

    pub fn declare_interface(&mut self, name: impl Into<Identifier>, loc: Loc, is_sealed: bool) {
        self.interfaces.push((name.into(), loc, is_sealed));
    }

    pub fn add_implementation(&mut self, implementer: impl Into<Identifier>, abstraction: impl Into<Identifier>, loc: Loc) {
        self.implementations.push((implementer.into(), abstraction.into(), loc));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Identifier {
    //TODO @mverleg: use this instead of string
    text: String,
}

impl Identifier {
    pub fn new(name: impl Into<String>) -> Self {
        // TODO validation happens here in the future
        Identifier { text: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.text
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Identifier::new(value)
    }
}

impl <'a> From<&'a str> for Identifier {
    fn from(value: &'a str) -> Self {
        Identifier::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ImplKey {
    implementer: Type,
    abstraction: Type,
}

#[derive(Debug)]
struct ImplInfo {
    declaration_loc: Loc,
}

#[derive(Debug)]
struct TypeContext {
    types_by_name: HashMap<Identifier, Rc<TypeInfo>>,
    implementations: HashMap<ImplKey, ImplInfo>
}

fn check_types(ast: &AST) -> Result<TypeContext, Vec<TypeErr>> {

    let mut errors = Vec::new();
    let types_by_name = collect_types(&ast, &mut errors);
    let implementations = collect_implementations(&ast, &types_by_name, &mut errors);

    if ! errors.is_empty() {
        return Err(errors)
    }
    Ok(TypeContext {
        types_by_name,
        implementations,
    })
}

fn collect_types(ast: &AST, errors: &mut Vec<TypeErr>) -> HashMap<Identifier, Rc<TypeInfo>> {
    let type_cnt = ast.structs.len() + ast.interfaces.len();
    let mut types_by_name: HashMap<Identifier, Rc<TypeInfo>> = HashMap::with_capacity(type_cnt);
    //let mut meta_for_type = HashMap::with_capacity(type_cnt);
    for (strct_name, loc) in &ast.structs {
        let kind = TypeKind::Struct;
        if let Some(existing) = types_by_name.get(strct_name) {
            errors.push(TypeErr::DoubleDeclaration {
                existing: existing.typ(),
                duplicate_kind: kind,
                duplicate_loc: loc.clone(),
            })
        } else {
            types_by_name.insert(strct_name.to_owned(), Rc::new(TypeInfo {
                id: types_by_name.len(),
                name: strct_name.clone(),
                kind,
                declaration_loc: loc.clone(),
            }));
        }
    }
    for (iface_name, loc, is_sealed) in &ast.interfaces {
        let kind = TypeKind::Interface { sealed: *is_sealed };
        if let Some(existing) = types_by_name.get(iface_name) {
            errors.push(TypeErr::DoubleDeclaration {
                existing: existing.typ(),
                duplicate_kind: kind,
                duplicate_loc: loc.clone(),
            })
        } else {
            types_by_name.insert(iface_name.to_owned(), Rc::new(TypeInfo {
                id: types_by_name.len(),
                name: iface_name.clone(),
                kind,
                declaration_loc: loc.clone(),
            }));
        }
    }
    types_by_name
}

fn collect_implementations(ast: &AST, types: &HashMap<Identifier, Rc<TypeInfo>>, errors: &mut Vec<TypeErr>) -> HashMap<ImplKey, ImplInfo> {
    let mut implementations: HashMap<ImplKey, ImplInfo> = HashMap::new();
    for (implementer_name, abstraction_name, impl_loc) in &ast.implementations {
        let implementer_type = match types.get(implementer_name) {
            Some(typ) => typ,
            None => {
                errors.push(TypeErr::NonExistentImplementer {
                    implementer: implementer_name.clone(),
                    abstraction: abstraction_name.clone(),
                    impl_loc: impl_loc.clone(),
                });
                continue
            }
        };
        let abstraction_type = match types.get(abstraction_name) {
            Some(typ) => typ,
            None => {
                errors.push(TypeErr::NonExistentAbstraction {
                    implementer: implementer_type.typ(),
                    abstraction: abstraction_name.clone(),
                    impl_loc: impl_loc.clone(),
                });
                continue
            }
        };
        if let TypeKind::Struct { .. } = abstraction_type.kind {
            errors.push(TypeErr::StructAbstraction {
                implementer: implementer_type.typ(),
                abstraction_struct: abstraction_type.typ(),
                impl_loc: impl_loc.clone(),
            });
            continue
        }
        if let TypeKind::Interface { sealed: true, .. } = abstraction_type.kind {
            //TODO @mverleg: mot sure yet about this one, let's start strict
            if ! matches!(implementer_type.kind, TypeKind::Struct { .. }) &&
                    ! matches!(implementer_type.kind, TypeKind::Interface { sealed: true, .. }) {
                panic!("sealed interface can only implement struct or another sealed interface");  //TODO @mverleg: real error if this restriction stays
            }
        }
        //TODO @mverleg: detect recursion
        let key = ImplKey { implementer: implementer_type.typ(), abstraction: abstraction_type.typ() };
        match implementations.entry(key) {
            Entry::Occupied(occupied) => {
                errors.push(TypeErr::DuplicateImplementation {
                    implementer: implementer_type.typ(),
                    abstraction: abstraction_type.typ(),
                    first_loc: (*occupied.get()).declaration_loc.clone(),
                    duplicate_loc: impl_loc.clone(),
                });
                continue
            }
            Entry::Vacant(vacant) => {
                let impl_info = ImplInfo {
                    declaration_loc: impl_loc.clone(),
                };
                vacant.insert(impl_info);
            }
        }
    }
    implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_ast() -> AST {
        let mut ast = AST::new();
        ast.declare_struct("int", Loc::dummy());
        ast.declare_struct("float", Loc::dummy());
        ast.declare_struct("String", Loc::dummy());
        ast.declare_struct("Password", Loc::dummy());
        ast.declare_interface("Display", Loc::dummy(), false);
        ast.declare_interface("Add", Loc::dummy(), false);
        ast.declare_interface("Sub", Loc::dummy(), false);
        ast.declare_interface("Number", Loc::dummy(), false);
        ast.declare_interface("TestSeal", Loc::dummy(), true);
        ast.add_implementation("int", "Add", Loc::dummy());
        ast.add_implementation("float", "Add", Loc::dummy());
        ast.add_implementation("int", "Sub", Loc::dummy());
        ast.add_implementation("float", "Sub", Loc::dummy());
        ast.add_implementation("Number", "Add", Loc::dummy());
        ast.add_implementation("Number", "Sub", Loc::dummy());
        ast.add_implementation("int", "Display", Loc::dummy());
        ast.add_implementation("float", "Display", Loc::dummy());
        ast.add_implementation("String", "Display", Loc::dummy());
        ast
    }

    #[test]
    fn typecheck_dummy_ast() {
        let ast = build_test_ast();
        check_types(&ast).unwrap();
    }

    #[test]
    fn duplicate_declaration_struct_struct_err() {
        let mut ast = build_test_ast();
        let new_loc = Loc::dummy();
        ast.declare_struct("Password", new_loc.clone());
        let errs = check_types(&ast).unwrap_err();
        assert_eq!(errs.len(), 1);
        let TypeErr::DoubleDeclaration { existing, duplicate_kind, duplicate_loc } = errs.into_iter().next().unwrap() else {
            panic!("wrong error")
        };
        assert_eq!(existing.name(), "Password");
        assert_eq!(duplicate_kind, TypeKind::Struct);
        assert_eq!(duplicate_loc, new_loc);
    }

    #[test]
    fn implement_struct_err() {
        let mut ast = build_test_ast();
        let new_loc = Loc::dummy();
        ast.add_implementation("int", "String", new_loc.clone());
        let errs = check_types(&ast).unwrap_err();
        assert_eq!(errs.len(), 1);
        let TypeErr::StructAbstraction { implementer, abstraction_struct, impl_loc } = errs.into_iter().next().unwrap() else {
            panic!("wrong error")
        };
        assert_eq!(implementer.name(), "int");
        assert_eq!(abstraction_struct.name(), "String");
        assert_eq!(impl_loc, new_loc);
    }

    #[test]
    fn non_existent_implementer_and_abstraction_err() {
        let mut ast = build_test_ast();
        let first_loc = Loc::dummy();
        let second_loc = Loc::dummy();
        ast.add_implementation("NonExistent", "TestSeal", first_loc.clone());
        ast.add_implementation("int", "NonExistent", second_loc.clone());
        let errs = check_types(&ast).unwrap_err();
        assert_eq!(errs.len(), 2);
        let mut err_iter = errs.into_iter();
        let TypeErr::NonExistentImplementer { implementer, abstraction, impl_loc } = err_iter.next().unwrap() else {
            panic!("wrong first error")
        };
        assert_eq!(implementer.name(), "NonExistent");
        assert_eq!(abstraction.name(), "TestSeal");
        assert_eq!(impl_loc, first_loc);
        let TypeErr::NonExistentAbstraction { implementer, abstraction, impl_loc } = err_iter.next().unwrap() else {
            panic!("wrong second error")
        };
        assert_eq!(implementer.name(), "int");
        assert_eq!(abstraction.name(), "NonExistent");
        assert_eq!(impl_loc, second_loc);
    }

    #[test]
    fn duplicate_impl_err() {
        let mut ast = build_test_ast();
        let first_loc = Loc::dummy();
        let second_loc = Loc::dummy();
        ast.add_implementation("int", "TestSeal", first_loc.clone());
        ast.add_implementation("int", "TestSeal", second_loc.clone());
        let errs = check_types(&ast).unwrap_err();
        assert_eq!(errs.len(), 1);
        let TypeErr::DuplicateImplementation { implementer, abstraction, first_loc, duplicate_loc } = errs.into_iter().next().unwrap() else {
            panic!("wrong error")
        };
        assert_eq!(implementer.name(), "int");
        assert_eq!(abstraction.name(), "TestSeal");
        assert_eq!(first_loc, first_loc);
        assert_eq!(duplicate_loc, second_loc);
    }

    #[test]
    fn self_impl_err() {
        let mut ast = build_test_ast();
        let new_loc = Loc::dummy();
        ast.add_implementation("TestSeal", "TestSeal", new_loc.clone());
        let errs = check_types(&ast).unwrap_err();
        assert_eq!(errs.len(), 1);
        let TypeErr::ImplementationCycle { cycle, impl_loc } = errs.into_iter().next().unwrap() else {
            panic!("wrong error")
        };
        assert_eq!(cycle.len(), 1);
        assert_eq!(cycle[0].name(), "TestSeal");
        assert_eq!(impl_loc, new_loc);
    }

    #[test]
    fn indirect_cycle_err() {
        let mut ast = build_test_ast();
        let main_loc = Loc::dummy();
        ast.add_implementation("Number", "TestSeal", Loc::dummy());
        ast.add_implementation("TestSeal", "Add", main_loc.clone());
        // Add <- Number <- TestSeal <- Add
        let errs = check_types(&ast).unwrap_err();
        assert_eq!(errs.len(), 1);
        let TypeErr::ImplementationCycle { cycle, impl_loc } = errs.into_iter().next().unwrap() else {
            panic!("wrong error")
        };
        assert_eq!(cycle.len(), 3);
        assert_eq!(cycle[0].name(), "TestSeal");
        assert_eq!(cycle[1].name(), "Add");
        assert_eq!(cycle[2].name(), "Number");
        assert_eq!(impl_loc, main_loc);
    }
}
