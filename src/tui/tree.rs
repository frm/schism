use crate::types::DiffFile;

/// A node in the hierarchy tree. Dir nodes own their children.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    /// Full path (file) or path prefix (dir)
    pub path: String,
    pub is_dir: bool,
    pub expanded: bool,
    /// App::files index for file nodes; usize::MAX for dirs
    pub file_index: usize,
    pub depth: usize,
    pub children: Vec<TreeNode>,
}

/// A flat entry for cursor navigation — just references into the tree.
#[derive(Debug, Clone)]
pub struct FlatNode {
    pub path: String,
    pub is_dir: bool,
    pub file_index: usize,
    pub depth: usize,
    pub expanded: bool,
}

pub fn build_tree(files: &[DiffFile]) -> Vec<TreeNode> {
    let mut root: Vec<TreeNode> = Vec::new();
    for (fi, file) in files.iter().enumerate() {
        let parts: Vec<&str> = file.path.split('/').collect();
        insert_node(&mut root, &parts, fi, 0);
    }
    sort_tree(&mut root);
    root
}

fn insert_node(nodes: &mut Vec<TreeNode>, parts: &[&str], file_index: usize, depth: usize) {
    let name = parts[0];
    let is_file = parts.len() == 1;

    if is_file {
        nodes.push(TreeNode {
            name: name.to_string(),
            path: build_path(nodes, name, depth),
            is_dir: false,
            expanded: false,
            file_index,
            depth,
            children: vec![],
        });
    } else {
        let pos = nodes.iter().position(|n| n.is_dir && n.name == name);
        let pos = pos.unwrap_or_else(|| {
            let path = build_path(nodes, name, depth);
            nodes.push(TreeNode {
                name: name.to_string(),
                path,
                is_dir: true,
                expanded: true,
                file_index: usize::MAX,
                depth,
                children: vec![],
            });
            nodes.len() - 1
        });
        insert_node(&mut nodes[pos].children, &parts[1..], file_index, depth + 1);
    }
}

fn build_path(siblings: &[TreeNode], name: &str, depth: usize) -> String {
    if depth == 0 {
        return name.to_string();
    }
    if let Some(sib) = siblings.first() {
        let prefix: Vec<&str> = sib.path.split('/').take(depth).collect();
        return format!("{}/{}", prefix.join("/"), name);
    }
    name.to_string()
}

fn sort_tree(nodes: &mut Vec<TreeNode>) {
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });
    for n in nodes.iter_mut() {
        if n.is_dir { sort_tree(&mut n.children); }
    }
}

pub fn flatten_tree(nodes: &[TreeNode]) -> Vec<FlatNode> {
    let mut out = Vec::new();
    flatten_walk(nodes, &mut out);
    out
}

fn flatten_walk(nodes: &[TreeNode], out: &mut Vec<FlatNode>) {
    for n in nodes {
        out.push(FlatNode {
            path: n.path.clone(),
            is_dir: n.is_dir,
            file_index: n.file_index,
            depth: n.depth,
            expanded: n.expanded,
        });
        if n.is_dir && n.expanded {
            flatten_walk(&n.children, out);
        }
    }
}

pub fn toggle_node_expanded(nodes: &mut Vec<TreeNode>, path: &str) {
    for n in nodes.iter_mut() {
        if n.path == path {
            n.expanded = !n.expanded;
            return;
        }
        if n.is_dir {
            toggle_node_expanded(&mut n.children, path);
        }
    }
}
