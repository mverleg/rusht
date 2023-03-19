
thread_local! {
    static TYPES: Vec<TypeKind> = Vec::new();
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

#[derive(Debug)]
pub struct Type {}

pub fn define() -> Type {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concrete_match() {
        todo!()
    }

    #[test]
    fn test_concrete_mismatch() {
        todo!()
    }
}
