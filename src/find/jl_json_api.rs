use ::std::cmp::Ordering;

use ::serde;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FSNode {
    pub name: String,
    pub base_name: String,
    pub extension: String,
    pub rel_path: String,
    pub canonical_path: String,
    pub is_dir: bool,
    pub is_link: bool,
    pub created_ts: (),  //TODO @mverleg:
    pub created_by: String,  //TODO @mverleg:
    pub changed_ts: (),  //TODO @mverleg:
    pub changed_age_sec: String,  //TODO @mverleg:
    pub changed_by: String,  //TODO @mverleg:
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
