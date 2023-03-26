
//TODO @mverleg: scopes

use std::collections::HashMap;
use std::collections::HashSet;

/// Source file location; dummy for now
#[derive(Debug)]
struct Loc {
    pub pos: u32,
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
    interfaces: Vec<(String, Loc)>,
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
        ast.declare_struct("int", loc);

        ast
    }

    #[test]
    fn test_add() {
        let ast = build_test_ast();
        check_types(&ast).unwrap();
    }
}