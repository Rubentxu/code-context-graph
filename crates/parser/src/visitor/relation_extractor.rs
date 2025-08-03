use crate::ast::{ASTNode, SimplifiedAST, ASTNodeType};
use crate::visitor::base::{ASTVisitor, VisitorContext, VisitResult};
use code_context_graph_core::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RelationInfo {
    pub from_entity: String,
    pub to_entity: String,
    pub relation_type: RelationType,
    pub source_location: crate::ast::NodeLocation,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationType {
    Inheritance,      // extends, implements
    Composition,      // has-a relationship
    Aggregation,      // uses
    Dependency,       // depends on
    Association,      // references
    CallsFunction,    // function call
    AccessesField,    // field access
    ImportModule,     // import/require
    DefinesType,      // type definition
}

pub struct RelationExtractor {
    relations: Vec<RelationInfo>,
    current_entity: Option<String>,
}

impl RelationExtractor {
    pub fn new() -> Self {
        Self {
            relations: Vec::new(),
            current_entity: None,
        }
    }

    fn extract_inheritance_relations(&mut self, node: &ASTNode) {
        if let Some(current) = &self.current_entity {
            // Check for extends relationship
            if let Some(extends) = node.get_metadata::<String>("extends") {
                let relation = RelationInfo {
                    from_entity: current.clone(),
                    to_entity: extends,
                    relation_type: RelationType::Inheritance,
                    source_location: node.location.clone(),
                    metadata: HashMap::new(),
                };
                self.relations.push(relation);
            }

            // Check for implements relationships
            if let Some(implements) = node.get_metadata::<Vec<String>>("implements") {
                for interface in implements {
                    let relation = RelationInfo {
                        from_entity: current.clone(),
                        to_entity: interface,
                        relation_type: RelationType::Inheritance,
                        source_location: node.location.clone(),
                        metadata: {
                            let mut map = HashMap::new();
                            map.insert("interface".to_string(), serde_json::Value::Bool(true));
                            map
                        },
                    };
                    self.relations.push(relation);
                }
            }

            // Check for parents (Kotlin style)
            if let Some(parents) = node.get_metadata::<Vec<String>>("parents") {
                for parent in parents {
                    let relation = RelationInfo {
                        from_entity: current.clone(),
                        to_entity: parent,
                        relation_type: RelationType::Inheritance,
                        source_location: node.location.clone(),
                        metadata: HashMap::new(),
                    };
                    self.relations.push(relation);
                }
            }
        }
    }

    fn extract_call_relations(&mut self, node: &ASTNode) {
        if let Some(current) = &self.current_entity {
            if let Some(called_function) = &node.name {
                let relation = RelationInfo {
                    from_entity: current.clone(),
                    to_entity: called_function.clone(),
                    relation_type: RelationType::CallsFunction,
                    source_location: node.location.clone(),
                    metadata: HashMap::new(),
                };
                self.relations.push(relation);
            }
        }
    }

    fn extract_member_access_relations(&mut self, node: &ASTNode) {
        if let Some(current) = &self.current_entity {
            if let Some(member_name) = &node.name {
                let relation = RelationInfo {
                    from_entity: current.clone(),
                    to_entity: member_name.clone(),
                    relation_type: RelationType::AccessesField,
                    source_location: node.location.clone(),
                    metadata: HashMap::new(),
                };
                self.relations.push(relation);
            }
        }
    }

    fn extract_import_relations(&mut self, node: &ASTNode) {
        if let Some(module_name) = &node.name {
            let relation = RelationInfo {
                from_entity: "current_module".to_string(), // This should be the current file/module
                to_entity: module_name.clone(),
                relation_type: RelationType::ImportModule,
                source_location: node.location.clone(),
                metadata: HashMap::new(),
            };
            self.relations.push(relation);
        }

        // Handle import details from metadata
        if let Some(imported_items) = node.get_metadata::<Vec<String>>("imported_items") {
            for item in imported_items {
                let relation = RelationInfo {
                    from_entity: "current_module".to_string(),
                    to_entity: item,
                    relation_type: RelationType::ImportModule,
                    source_location: node.location.clone(),
                    metadata: HashMap::new(),
                };
                self.relations.push(relation);
            }
        }
    }

    fn get_current_scope_entity(&self, context: &VisitorContext) -> Option<String> {
        if !context.current_scope.is_empty() {
            Some(context.current_scope_path())
        } else {
            None
        }
    }
}

impl ASTVisitor for RelationExtractor {
    type Output = Vec<RelationInfo>;

    fn visit_ast(&mut self, ast: &SimplifiedAST, context: &mut VisitorContext) -> Result<Self::Output> {
        self.visit_node(&ast.root, context)?;
        Ok(self.relations.clone())
    }

    fn visit_node(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<VisitResult> {
        match &node.node_type {
            ASTNodeType::ClassDeclaration | 
            ASTNodeType::InterfaceDeclaration |
            ASTNodeType::EnumDeclaration => {
                if let Some(name) = &node.name {
                    context.push_scope(name.clone());
                    self.current_entity = Some(context.current_scope_path());
                    self.extract_inheritance_relations(node);
                }
            }
            ASTNodeType::FunctionDeclaration |
            ASTNodeType::MethodDeclaration => {
                if let Some(name) = &node.name {
                    let previous_entity = self.current_entity.clone();
                    self.current_entity = Some(format!("{}::{}", 
                        context.current_scope_path(), name));
                    
                    // Visit children to find calls and dependencies within this function
                    let result = self.visit_children(node, context)?;
                    
                    self.current_entity = previous_entity;
                    return Ok(result);
                }
            }
            ASTNodeType::CallExpression => {
                self.extract_call_relations(node);
            }
            ASTNodeType::MemberExpression => {
                self.extract_member_access_relations(node);
            }
            ASTNodeType::ImportDeclaration => {
                self.extract_import_relations(node);
            }
            _ => {}
        }

        // Continue visiting children
        let result = self.visit_children(node, context)?;
        
        // Pop scope if we pushed one
        match &node.node_type {
            ASTNodeType::ClassDeclaration | 
            ASTNodeType::InterfaceDeclaration | 
            ASTNodeType::EnumDeclaration => {
                context.pop_scope();
                self.current_entity = self.get_current_scope_entity(context);
            }
            _ => {}
        }

        Ok(result)
    }
}

impl Default for RelationExtractor {
    fn default() -> Self {
        Self::new()
    }
}