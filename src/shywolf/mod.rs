
//TODO @mverleg: scopes

use std::collections::hash_map::OccupiedError;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static TYPE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static DUMMY_LOC_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Source file location; dummy for now
#[derive(Debug)]
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
    DoubleDeclaration { existing: Type, duplicate: Loc }
}

#[derive(Debug)]
struct Type {
    pub id: usize,
}

impl Type {
    pub fn new() -> Type {
        Type { id: TYPE_COUNTER.fetch_add(1, Ordering::AcqRel) }
    }
}

#[derive(Debug)]
enum TypeKind {
    Struct,
    Interface { sealed: bool, },
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
struct TypeContext {}

fn check_types(ast: &AST) -> Result<TypeContext, Vec<TypeErr>> {
    let mut errors = Vec::new();
    let type_cnt = ast.structs.len() + ast.interfaces.len();
    let mut types_by_name = HashMap::with_capacity(type_cnt);
    //let mut meta_for_type = HashMap::with_capacity(type_cnt);
    for (strct_name, loc) in ast.structs {
        let new_typ = Type::new();
        match types_by_name.try_insert(strct_name, new_typ) {
            Ok(_) => {}
            Err(existing_entry) => {
                errors.push(TypeErr::DoubleDeclaration { existing: existing_entry.value, duplicate: loc })
            }
        }
    }
    todo!()
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
    fn test_add() {
        let ast = build_test_ast();
        check_types(&ast).unwrap();
    }
}