use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentIndex {
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildIndex {
    pub recipes: Vec<Recipe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub repo: String,
    pub branch: Option<String>,
    pub name: String,
    pub labels: Option<Vec<String>>,
    pub schemas: Vec<String>,
    pub dependencies: Option<Vec<String>>,
    #[serde(rename = "reverseDependencies")]
    pub reverse_dependencies: Option<Vec<String>>,
    pub license: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CatalogNode {
    pub path: PathBuf,
    pub parent: Option<ParentIndex>,
    pub child: Option<ChildIndex>,
    pub children: HashMap<String, CatalogNode>,
}

impl CatalogNode {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            parent: None,
            child: None,
            children: HashMap::new(),
        }
    }
}

/// Load a catalog node recursively from the given directory.
pub fn load_catalog<P: AsRef<Path>>(dir: P) -> io::Result<CatalogNode> {
    let dir = dir.as_ref();
    let mut node = CatalogNode::new(dir.to_path_buf());

    let index_path = dir.join("index.json");
    if index_path.exists() {
        let s = fs::read_to_string(&index_path)?;
        // Try to detect which index it is by inspecting JSON keys
        let v: Value = serde_json::from_str(&s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if v.get("categories").is_some() {
            let p: ParentIndex = serde_json::from_value(v).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            node.parent = Some(p);
        } else if v.get("recipes").is_some() {
            let c: ChildIndex = serde_json::from_value(v).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            node.child = Some(c);
        }
    }

    // Recurse into subdirectories
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Recurse and insert under directory name
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if name.is_empty() { continue; }
            match load_catalog(&path) {
                Ok(child_node) => { node.children.insert(name, child_node); }
                Err(_) => { /* ignore subdir errors */ }
            }
        }
    }

    Ok(node)
}

/// Collect all recipes in the catalog subtree into a flat Vec.
pub fn collect_all_recipes(node: &CatalogNode) -> Vec<Recipe> {
    let mut out = Vec::new();
    if let Some(child) = &node.child {
        out.extend(child.recipes.clone());
    }
    for c in node.children.values() {
        out.extend(collect_all_recipes(c));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_load_catalog_parent_and_child() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();

        // write parent index.json
        let parent = r#"{ "categories": [{ "key": "key1", "name": "Key 1" }] }"#;
        fs::write(root.join("index.json"), parent).unwrap();

        // create child dir
        let child_dir = root.join("key1");
        fs::create_dir_all(&child_dir).unwrap();
        let child = r#"{ "recipes": [{ "repo": "rime/example", "name": "Example", "schemas": ["luna_pinyin"] }] }"#;
        fs::write(child_dir.join("index.json"), child).unwrap();

        let catalog = load_catalog(root).expect("load");
        assert!(catalog.parent.is_some());
        assert!(catalog.children.contains_key("key1"));
        let child_node = &catalog.children["key1"];
        assert!(child_node.child.is_some());
    }
}
