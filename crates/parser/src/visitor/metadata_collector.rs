use crate::ast::{ASTNode, SimplifiedAST, ASTNodeType};
use crate::visitor::base::{ASTVisitor, VisitorContext, VisitResult};
use code_context_graph_core::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CodeMetrics {
    pub total_lines: u32,
    pub total_files: u32,
    pub classes_count: u32,
    pub functions_count: u32,
    pub methods_count: u32,
    pub variables_count: u32,
    pub complexity_score: f64,
    pub language_specific: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct EntityMetadata {
    pub entity_name: String,
    pub entity_type: String,
    pub location: crate::ast::NodeLocation,
    pub cyclomatic_complexity: u32,
    pub lines_of_code: u32,
    pub parameters_count: u32,
    pub dependencies: Vec<String>,
    pub attributes: HashMap<String, serde_json::Value>,
}

pub struct MetadataCollector {
    metrics: CodeMetrics,
    entity_metadata: Vec<EntityMetadata>,
    current_complexity: u32,
}

impl MetadataCollector {
    pub fn new() -> Self {
        Self {
            metrics: CodeMetrics {
                total_lines: 0,
                total_files: 0,
                classes_count: 0,
                functions_count: 0,
                methods_count: 0,
                variables_count: 0,
                complexity_score: 0.0,
                language_specific: HashMap::new(),
            },
            entity_metadata: Vec::new(),
            current_complexity: 1, // Base complexity is 1
        }
    }

    fn calculate_cyclomatic_complexity(&mut self, node: &ASTNode) -> u32 {
        let mut complexity = 1; // Base complexity

        // Add complexity for control flow structures
        match &node.node_type {
            ASTNodeType::IfStatement => complexity += 1,
            ASTNodeType::ForStatement => complexity += 1,
            ASTNodeType::WhileStatement => complexity += 1,
            _ => {}
        }

        // Recursively calculate complexity for children
        for child in &node.children {
            complexity += self.calculate_cyclomatic_complexity(child);
        }

        complexity
    }

    fn calculate_lines_of_code(&self, node: &ASTNode) -> u32 {
        let location = &node.location;
        (location.end_line - location.start_line) + 1
    }

    fn count_parameters(&self, node: &ASTNode) -> u32 {
        // Check metadata for parameter information
        if let Some(params) = node.get_metadata::<Vec<String>>("parameters") {
            params.len() as u32
        } else {
            // Count parameter nodes in children
            node.children.iter()
                .filter(|child| matches!(child.node_type, ASTNodeType::VariableDeclaration))
                .count() as u32
        }
    }

    fn extract_dependencies(&self, node: &ASTNode) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Extract from imports
        if let Some(imports) = node.get_metadata::<Vec<String>>("imports") {
            dependencies.extend(imports);
        }

        // Extract from type annotations
        if let Some(types) = node.get_metadata::<Vec<String>>("types") {
            dependencies.extend(types);
        }

        // Extract from function calls
        for child in &node.children {
            if matches!(child.node_type, ASTNodeType::CallExpression) {
                if let Some(name) = &child.name {
                    dependencies.push(name.clone());
                }
            }
        }

        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    fn collect_entity_metadata(&mut self, node: &ASTNode, entity_type: &str) {
        if let Some(name) = &node.name {
            let complexity = self.calculate_cyclomatic_complexity(node);
            let lines_of_code = self.calculate_lines_of_code(node);
            let parameters_count = self.count_parameters(node);
            let dependencies = self.extract_dependencies(node);

            let metadata = EntityMetadata {
                entity_name: name.clone(),
                entity_type: entity_type.to_string(),
                location: node.location.clone(),
                cyclomatic_complexity: complexity,
                lines_of_code,
                parameters_count,
                dependencies,
                attributes: node.metadata.clone(),
            };

            self.entity_metadata.push(metadata);
        }
    }

    fn update_metrics_for_node(&mut self, node: &ASTNode) {
        match &node.node_type {
            ASTNodeType::ClassDeclaration => {
                self.metrics.classes_count += 1;
                self.collect_entity_metadata(node, "class");
            }
            ASTNodeType::FunctionDeclaration => {
                self.metrics.functions_count += 1;
                self.collect_entity_metadata(node, "function");
            }
            ASTNodeType::MethodDeclaration => {
                self.metrics.methods_count += 1;
                self.collect_entity_metadata(node, "method");
            }
            ASTNodeType::VariableDeclaration => {
                self.metrics.variables_count += 1;
            }
            _ => {}
        }
    }

    fn collect_language_specific_metadata(&mut self, ast: &SimplifiedAST) {
        match ast.language {
            code_context_graph_core::Language::Python => {
                // Python-specific metrics
                let decorators_count = self.count_node_type(ast, &ASTNodeType::Decorator);
                let comprehensions_count = self.count_node_type(ast, &ASTNodeType::Comprehension);
                
                self.metrics.language_specific.insert(
                    "decorators_count".to_string(),
                    serde_json::Value::Number(decorators_count.into())
                );
                self.metrics.language_specific.insert(
                    "comprehensions_count".to_string(),
                    serde_json::Value::Number(comprehensions_count.into())
                );
            }
            code_context_graph_core::Language::Java => {
                // Java-specific metrics
                let annotations_count = self.count_node_type(ast, &ASTNodeType::Annotation);
                let interfaces_count = self.count_node_type(ast, &ASTNodeType::InterfaceDeclaration);
                
                self.metrics.language_specific.insert(
                    "annotations_count".to_string(),
                    serde_json::Value::Number(annotations_count.into())
                );
                self.metrics.language_specific.insert(
                    "interfaces_count".to_string(),
                    serde_json::Value::Number(interfaces_count.into())
                );
            }
            code_context_graph_core::Language::JavaScript => {
                // JavaScript-specific metrics
                let arrow_functions = self.count_arrow_functions(ast);
                let async_functions = self.count_async_functions(ast);
                
                self.metrics.language_specific.insert(
                    "arrow_functions_count".to_string(),
                    serde_json::Value::Number(arrow_functions.into())
                );
                self.metrics.language_specific.insert(
                    "async_functions_count".to_string(),
                    serde_json::Value::Number(async_functions.into())
                );
            }
            code_context_graph_core::Language::Kotlin => {
                // Kotlin-specific metrics
                let data_classes = self.count_data_classes(ast);
                let suspend_functions = self.count_suspend_functions(ast);
                
                self.metrics.language_specific.insert(
                    "data_classes_count".to_string(),
                    serde_json::Value::Number(data_classes.into())
                );
                self.metrics.language_specific.insert(
                    "suspend_functions_count".to_string(),
                    serde_json::Value::Number(suspend_functions.into())
                );
            }
            _ => {}
        }
    }

    fn count_node_type(&self, ast: &SimplifiedAST, node_type: &ASTNodeType) -> u32 {
        self.count_node_type_recursive(&ast.root, node_type)
    }

    fn count_node_type_recursive(&self, node: &ASTNode, target_type: &ASTNodeType) -> u32 {
        let mut count = if &node.node_type == target_type { 1 } else { 0 };
        
        for child in &node.children {
            count += self.count_node_type_recursive(child, target_type);
        }
        
        count
    }

    fn count_arrow_functions(&self, ast: &SimplifiedAST) -> u32 {
        self.count_functions_with_attribute(&ast.root, "arrow", true)
    }

    fn count_async_functions(&self, ast: &SimplifiedAST) -> u32 {
        self.count_functions_with_attribute(&ast.root, "async", true)
    }

    fn count_data_classes(&self, ast: &SimplifiedAST) -> u32 {
        self.count_classes_with_modifier(&ast.root, "data")
    }

    fn count_suspend_functions(&self, ast: &SimplifiedAST) -> u32 {
        self.count_functions_with_modifier(&ast.root, "suspend")
    }

    fn count_functions_with_attribute(&self, node: &ASTNode, attribute: &str, expected_value: bool) -> u32 {
        let mut count = 0;
        
        if matches!(node.node_type, ASTNodeType::FunctionDeclaration | ASTNodeType::MethodDeclaration) {
            if let Some(value) = node.get_metadata::<bool>(attribute) {
                if value == expected_value {
                    count += 1;
                }
            }
        }
        
        for child in &node.children {
            count += self.count_functions_with_attribute(child, attribute, expected_value);
        }
        
        count
    }

    fn count_classes_with_modifier(&self, node: &ASTNode, modifier: &str) -> u32 {
        let mut count = 0;
        
        if matches!(node.node_type, ASTNodeType::ClassDeclaration) {
            if let Some(modifiers) = node.get_metadata::<Vec<String>>("modifiers") {
                if modifiers.contains(&modifier.to_string()) {
                    count += 1;
                }
            }
        }
        
        for child in &node.children {
            count += self.count_classes_with_modifier(child, modifier);
        }
        
        count
    }

    fn count_functions_with_modifier(&self, node: &ASTNode, modifier: &str) -> u32 {
        let mut count = 0;
        
        if matches!(node.node_type, ASTNodeType::FunctionDeclaration | ASTNodeType::MethodDeclaration) {
            if let Some(modifiers) = node.get_metadata::<Vec<String>>("modifiers") {
                if modifiers.contains(&modifier.to_string()) {
                    count += 1;
                }
            }
        }
        
        for child in &node.children {
            count += self.count_functions_with_modifier(child, modifier);
        }
        
        count
    }
}

impl ASTVisitor for MetadataCollector {
    type Output = (CodeMetrics, Vec<EntityMetadata>);

    fn visit_ast(&mut self, ast: &SimplifiedAST, context: &mut VisitorContext) -> Result<Self::Output> {
        // Initialize file count
        self.metrics.total_files = 1;
        
        // Calculate total lines from the source
        self.metrics.total_lines = context.source.lines().count() as u32;
        
        // Visit all nodes to collect metrics
        self.visit_node(&ast.root, context)?;
        
        // Collect language-specific metadata
        self.collect_language_specific_metadata(ast);
        
        // Calculate overall complexity score
        let total_entities = self.metrics.classes_count + self.metrics.functions_count + self.metrics.methods_count;
        if total_entities > 0 {
            let total_complexity: u32 = self.entity_metadata.iter()
                .map(|e| e.cyclomatic_complexity)
                .sum();
            self.metrics.complexity_score = total_complexity as f64 / total_entities as f64;
        }
        
        Ok((self.metrics.clone(), self.entity_metadata.clone()))
    }

    fn visit_node(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<VisitResult> {
        // Update metrics for current node
        self.update_metrics_for_node(node);
        
        // Continue visiting children
        self.visit_children(node, context)
    }
}

impl Default for MetadataCollector {
    fn default() -> Self {
        Self::new()
    }
}