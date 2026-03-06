// crates/sup/src/output.rs
use sup_core::models::{Node, NodeType, TaskStatus};

pub fn print_node_tree(nodes: &[Node], indent: usize) {
    for node in nodes {
        let prefix = "  ".repeat(indent);
        let icon = match &node.node_type {
            NodeType::Task => node.status.as_ref()
                .map(|s| s.icon())
                .unwrap_or("☐"),
            t => t.icon(),
        };
        let tag_str = if node.tags.is_empty() {
            String::new()
        } else {
            format!("  \x1b[2m#{}\x1b[0m", node.tags.join(" #"))
        };
        let priority_str = if node.node_type == NodeType::Task {
            match &node.priority {
                Some(p) => format!(" [{}]", p.as_str().to_uppercase()),
                None => String::new(),
            }
        } else {
            String::new()
        };
        println!("{}{} {}{}{}", prefix, icon, node.content, priority_str, tag_str);
        print_node_tree(&node.children, indent + 1);
    }
}
