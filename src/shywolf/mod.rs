
//TODO @mverleg: scopes

use std::collections::HashMap;
use std::collections::HashSet;

/// Source file location; dummy for now
#[derive(Debug)]
pub struct Loc {
    pub pos: u32,
}

#[derive(Debug)]
pub enum TypeErr {
    //
}

#[derive(Debug)]
pub struct Type {
    //
}

#[derive(Debug)]
struct AST {
    impls: Vec<(Type, Type, Loc)>,
}

#[derive(Debug)]
struct TypeContext {}

