use code_context_graph_viz::mermaid::ClassDiagramExporter;
use code_context_graph_parser::ast::{ASTNode, ASTNodeType, NodeLocation, SimplifiedAST};
use code_context_graph_core::Language;

fn loc() -> NodeLocation { NodeLocation::new(1,0,1,1,0,1) }

#[test]
fn generates_inheritance_diagram() {
    // Build a minimal AST: root -> class A, class B (extends A)
    let mut root = ASTNode::new(ASTNodeType::Program, None, loc());

    let class_a = ASTNode::new(ASTNodeType::ClassDeclaration, Some("A".into()), loc());

    let mut class_b = ASTNode::new(ASTNodeType::ClassDeclaration, Some("B".into()), loc());
    class_b.add_metadata("extends", "A");

    root.add_child(class_a);
    root.add_child(class_b);

    let ast = SimplifiedAST::new(root, Language::Java, "");

    let mermaid = ClassDiagramExporter::from_ast(&ast);

    let expected_lines = vec![
        "classDiagram",
        "class A",
        "class B",
        "A <|-- B",
    ];

    for line in expected_lines {
        assert!(mermaid.contains(line), "missing line: {}\noutput:\n{}", line, mermaid);
    }
}
