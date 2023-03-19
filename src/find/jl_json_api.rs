use ::std::cmp::Ordering;

use ::serde;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FSNode {
    pub name: String,
    pub safe_name: String,
    pub base_name: String,
    pub extension: String,
    pub rel_path: String,
    pub canonical_path: String,
    pub is_dir: bool,
    pub is_link: bool,
    pub size_b: u64,
    pub size_mb: u64,
    pub created_ts: u64,
    //pub created_by: String,
    pub changed_ts: u64,
    pub changed_age_sec: u64,
    //pub changed_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    //TODO @mverleg: permissions
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
