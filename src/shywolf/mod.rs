
//TODO @mverleg: scopes

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};

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
    //
}

#[derive(Debug)]
struct Type {
    pub id: usize,
    name: String,
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
    implementations: Vec<(Type, Type, Loc)>,
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
}

#[derive(Debug)]
struct TypeContext {}

fn check_types(ast: &AST) -> Result<TypeContext, Vec<TypeErr>> {
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
        ast
    }

    #[test]
    fn test_add() {
        let ast = build_test_ast();
        check_types(&ast).unwrap();
    }
}