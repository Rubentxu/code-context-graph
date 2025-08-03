use crate::ast::{ASTNode, SimplifiedAST, ASTNodeType};
use crate::visitor::base::{ASTVisitor, VisitorContext, VisitResult};
use code_context_graph_core::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub name: String,
    pub entity_type: EntityType,
    pub location: crate::ast::NodeLocation,
    pub visibility: Option<String>,
    pub modifiers: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityType {
    Class,
    Interface,
    Function,
    Method,
    Variable,
    Constant,
    Module,
    Enum,
    Trait,
}

pub struct EntityExtractor {
    entities: Vec<EntityInfo>,
}

impl EntityExtractor {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    fn extract_modifiers(&self, node: &ASTNode) -> Vec<String> {
        node.get_metadata::<Vec<String>>("modifiers").unwrap_or_default()
    }

    fn extract_visibility(&self, node: &ASTNode) -> Option<String> {
        node.get_metadata::<String>("visibility")
    }

    fn create_entity_info(&self, node: &ASTNode, entity_type: EntityType) -> Option<EntityInfo> {
        let name = node.name.clone()?;
        
        Some(EntityInfo {
            name,
            entity_type,
            location: node.location.clone(),
            visibility: self.extract_visibility(node),
            modifiers: self.extract_modifiers(node),
            metadata: node.metadata.clone(),
        })
    }
}

impl ASTVisitor for EntityExtractor {
    type Output = Vec<EntityInfo>;

    fn visit_ast(&mut self, ast: &SimplifiedAST, context: &mut VisitorContext) -> Result<Self::Output> {
        self.visit_node(&ast.root, context)?;
        Ok(self.entities.clone())
    }

    fn visit_node(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<VisitResult> {
        match &node.node_type {
            ASTNodeType::ClassDeclaration => {
                if let Some(entity) = self.create_entity_info(node, EntityType::Class) {
                    context.push_scope(entity.name.clone());
                    self.entities.push(entity);
                }
            }
            ASTNodeType::InterfaceDeclaration => {
                if let Some(entity) = self.create_entity_info(node, EntityType::Interface) {
                    context.push_scope(entity.name.clone());
                    self.entities.push(entity);
                }
            }
            ASTNodeType::FunctionDeclaration => {
                let entity_type = if context.current_scope.is_empty() {
                    EntityType::Function
                } else {
                    EntityType::Method
                };
                if let Some(entity) = self.create_entity_info(node, entity_type) {
                    self.entities.push(entity);
                }
            }
            ASTNodeType::MethodDeclaration => {
                if let Some(entity) = self.create_entity_info(node, EntityType::Method) {
                    self.entities.push(entity);
                }
            }
            ASTNodeType::VariableDeclaration => {
                if let Some(entity) = self.create_entity_info(node, EntityType::Variable) {
                    self.entities.push(entity);
                }
            }
            ASTNodeType::EnumDeclaration => {
                if let Some(entity) = self.create_entity_info(node, EntityType::Enum) {
                    context.push_scope(entity.name.clone());
                    self.entities.push(entity);
                }
            }
            ASTNodeType::Module => {
                if let Some(entity) = self.create_entity_info(node, EntityType::Module) {
                    context.push_scope(entity.name.clone());
                    self.entities.push(entity);
                }
            }
            _ => {}
        }

        // Continue visiting children
        let result = self.visit_children(node, context)?;
        
        // Pop scope if we pushed one
        match &node.node_type {
            ASTNodeType::ClassDeclaration | 
            ASTNodeType::InterfaceDeclaration | 
            ASTNodeType::EnumDeclaration | 
            ASTNodeType::Module => {
                context.pop_scope();
            }
            _ => {}
        }

        Ok(result)
    }
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new()
    }
}