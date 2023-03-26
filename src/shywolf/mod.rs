
//TODO @mverleg: scopes

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

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

#[derive(Debug)]
enum TypeErr {
    DoubleDeclaration { existing: Type, duplicate_kind: TypeKind, duplicate_loc: Loc },
    DoubleImpl {},
}

#[derive(Debug)]
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
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
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
    name: String,
    kind: TypeKind,
}

#[derive(Debug)]
struct TypeCache {
    contents: HashMap<String, Rc<TypeInfo>>,
}

impl TypeCache {
    pub fn new() -> Self {
        TypeCache { contents: HashMap::new() }
    }

    /// Does not check if existing value is the same/compatible with new one
    pub fn add_or_get(&mut self, name: impl Into<String>, kind: TypeKind) -> Type {
        let name = name.into();
        match self.contents.entry(name) {
            Entry::Occupied(occupied) =>
                Type::of(occupied.get().clone()),
            Entry::Vacant(vacant) => {
                let info = Rc::new(TypeInfo { id: self.contents.len(), name, kind, });
                vacant.insert(info.clone());
                Type::of(info)
            }
        }
    }
}

#[derive(Debug)]
struct AST {
    structs: Vec<(String, Loc)>,
    interfaces: Vec<(String, Loc, bool)>,
    implementations: Vec<(String, String, Loc)>,
}

impl AST {
    pub fn new() -> Self {
        AST {
            structs: Vec::new(),
            interfaces: Vec::new(),
            implementations: Vec::new(),
        }
    }

    pub fn declare_struct(&mut self, name: impl Into<String>, loc: Loc) {
        self.structs.push((name.into(), loc));
    }

    pub fn declare_interface(&mut self, name: impl Into<String>, loc: Loc, is_sealed: bool) {
        self.interfaces.push((name.into(), loc, is_sealed));
    }

    pub fn add_implementation(&mut self, implementer: impl Into<String>, abstraction: impl Into<String>, loc: Loc) {
        self.implementations.push((implementer.into(), abstraction.into(), loc));
    }
}

#[derive(Debug)]
struct TypeContext {
    types_by_name: HashMap<String, Type>,
}

fn check_types(ast: &AST, x: &mut TypeCache) -> Result<TypeContext, Vec<TypeErr>> {
    let mut errors = Vec::new();
    let types_by_name = collect_types(&ast, &mut errors);

    Ok(TypeContext {
        types_by_name,
    })
}

fn collect_types(ast: &AST, errors: &mut Vec<TypeErr>) -> HashMap<String, Type> {
    let type_cnt = ast.structs.len() + ast.interfaces.len();
    let mut types_by_name = HashMap::with_capacity(type_cnt);
    //let mut meta_for_type = HashMap::with_capacity(type_cnt);
    for (strct_name, loc) in &ast.structs {
        if let Some(existing) = types_by_name.get(strct_name) {
            errors.push(TypeErr::DoubleDeclaration { existing: existing_entry.value, duplicate_kind: TypeKind::Struct, duplicate_loc: loc.clone() })
        } else {
            types_by_name.insert(strct_name.to_owned(), );
        }
    }
    for (iface_name, loc, is_sealed) in &ast.interfaces {
        if let Some(existing) = types_by_name.get(strct_name) {
            errors.push(TypeErr::DoubleDeclaration { existing: existing_entry.value, duplicate_kind: TypeKind::Interface { sealed: *is_sealed }, duplicate_loc: loc.clone() })
        } else {
            types_by_name.insert(strct_name.to_owned(), );
        }
    }
    types_by_name
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
        ast.add_implementation("int", "Add", Loc::dummy());
        ast.add_implementation("float", "Add", Loc::dummy());
        ast.add_implementation("int", "Display", Loc::dummy());
        ast.add_implementation("float", "Display", Loc::dummy());
        ast.add_implementation("string", "Display", Loc::dummy());
        ast
    }

    #[test]
    fn typecheck_dummy_ast() {
        let ast = build_test_ast();
        check_types(&ast, &mut TypeCache::new()).unwrap();
    }

    #[test]
    fn duplicate_declaration_struct_struct() {
        let mut ast = build_test_ast();
        let new_loc = Loc::dummy();
        ast.declare_struct("Password", new_loc);
        let errs = check_types(&ast, &mut TypeCache::new()).unwrap_err();
        assert_eq!(errs.len(), 1);
        let TypeErr::DoubleDeclaration { existing, duplicate_kind, duplicate_loc } = errs.into_iter().next().unwrap() else {
            panic!("wrong error")
        };
        assert_eq!(existing.name(), "");
        assert_eq!(duplicate_kind, TypeKind::Struct);
        assert_eq!(duplicate_loc, new_loc);
    }
}