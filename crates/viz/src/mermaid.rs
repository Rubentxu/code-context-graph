use code_context_graph_parser::ast::{SimplifiedAST, ASTNodeType};

pub struct ClassDiagramExporter;

impl ClassDiagramExporter {
    pub fn from_ast(ast: &SimplifiedAST) -> String {
        Self::from_ast_with_filter(ast, None)
    }

    pub fn from_ast_with_filter(ast: &SimplifiedAST, filter: Option<&[String]>) -> String {
        let mut out = String::new();
        out.push_str("classDiagram\n");

        // Collect class names and relations
        let mut classes: Vec<String> = Vec::new();
        let mut inherits: Vec<(String, String)> = Vec::new(); // (parent, child)

        fn walk(node: &code_context_graph_parser::ast::ASTNode, classes: &mut Vec<String>, inherits: &mut Vec<(String, String)>) {
            match node.node_type {
                ASTNodeType::ClassDeclaration => {
                    if let Some(name) = &node.name {
                        if !classes.iter().any(|c| c == name) {
                            classes.push(name.clone());
                        }
                        if let Some(parent) = node.get_metadata::<String>("extends") {
                            inherits.push((parent, name.clone()));
                        }
                    }
                }
                _ => {}
            }
            for child in &node.children {
                walk(child, classes, inherits);
            }
        }

        walk(&ast.root, &mut classes, &mut inherits);

        // Apply filter if provided
        let keep: Box<dyn Fn(&String) -> bool> = if let Some(list) = filter {
            let set: std::collections::HashSet<String> = list.iter().cloned().collect();
            Box::new(move |name: &String| set.contains(name))
        } else {
            Box::new(|_| true)
        };

        for c in classes.iter().filter(|c| keep(c)) {
            out.push_str(&format!("class {}\n", c));
        }
        for (parent, child) in inherits.into_iter() {
            if keep(&parent) && keep(&child) {
                out.push_str(&format!("{} <|-- {}\n", parent, child));
            }
        }

        out
    }
}
