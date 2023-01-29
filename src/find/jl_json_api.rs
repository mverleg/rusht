use ::std::cmp::Ordering;

#[derive(Debug, Clone)]
struct FSNode {
    name: String,
    base_name: String,
    extension: String,
    rel_path: String,
    canonical_path: String,
    is_dir: bool,
    is_link: bool,
    created: (),  //TODO @mverleg:
    changed: (),  //TODO @mverleg:
}

impl PartialEq for FSNode {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_path.eq(&other.canonical_path)
    }
}

impl Eq for FSNode {}

impl PartialOrd for FSNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.canonical_path.partial_cmp(&other.canonical_path)
    }
}

impl Ord for FSNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.canonical_path.cmp(&other.canonical_path)
    }
}
