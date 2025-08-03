# Connascence Analysis Guide

## Overview

Connascence is a software quality metric that measures the degree and type of coupling between software components. Code Context Graph implements comprehensive connascence detection to identify refactoring opportunities and maintain code quality.

## Connascence Theory

### Definition

Connascence exists between two software elements A and B if changing A requires a corresponding change in B to maintain system correctness. The stronger the connascence, the more tightly coupled the elements.

### Properties

Every form of connascence has three properties:

1. **Strength**: How difficult it is to refactor
2. **Locality**: How close the coupled elements are
3. **Degree**: How many elements are involved

## Types of Connascence

### Static Connascence (Detectable at Compile Time)

#### Connascence of Name (CoN)
Multiple components must agree on the name of an entity.

**Example (Python):**
```python
# Function definition
def calculate_payment(amount, tax_rate):
    return amount * (1 + tax_rate)

# Function call - must use exact name
result = calculate_payment(100, 0.08)  # CoN with function name
```

**Strength**: Weak (easy to refactor with IDE)
**Refactoring**: Use IDE rename refactoring

#### Connascence of Type (CoT)
Multiple components must agree on the type of an entity.

**Example (Java):**
```java
// Method expects specific type
public void processUser(User user) {
    // Implementation
}

// Caller must provide compatible type
processUser(new AdminUser());  // CoT with User type hierarchy
```

**Strength**: Weak to Medium
**Refactoring**: Extract interfaces, use generics

#### Connascence of Meaning (CoM)
Multiple components must agree on the meaning of specific values.

**Example (JavaScript):**
```javascript
// Status codes with implicit meaning
function getUserStatus(userId) {
    // Returns: 0=inactive, 1=active, 2=banned
    return userDatabase.getStatus(userId);
}

// Caller must know the meaning
if (getUserStatus(123) === 1) {  // CoM with status value
    // User is active
}
```

**Strength**: Medium
**Refactoring**: Use enums, constants, or symbolic names

#### Connascence of Position (CoP)
Multiple components must agree on the order of values.

**Example (Python):**
```python
# Function with positional parameters
def create_user(name, email, age, is_admin):
    return User(name, email, age, is_admin)

# Caller must provide arguments in correct order
user = create_user("John", "john@example.com", 25, False)  # CoP
```

**Strength**: Medium to Strong
**Refactoring**: Use named parameters, data classes

#### Connascence of Algorithm (CoA)
Multiple components must agree on a particular algorithm.

**Example (Python):**
```python
# Encryption function
def encrypt_password(password):
    return hashlib.sha256(password.encode()).hexdigest()

# Verification function must use same algorithm
def verify_password(password, hash_value):
    return hashlib.sha256(password.encode()).hexdigest() == hash_value  # CoA
```

**Strength**: Strong
**Refactoring**: Extract algorithm to shared component

### Dynamic Connascence (Detectable Only at Runtime)

#### Connascence of Execution (CoE)
The order of execution of multiple components is important.

**Example (Python):**
```python
class DatabaseConnection:
    def __init__(self):
        self.connection = None
    
    def connect(self):
        self.connection = create_connection()
    
    def query(self, sql):
        # Must call connect() before query()
        return self.connection.execute(sql)  # CoE with connect()

# Usage
db = DatabaseConnection()
db.connect()  # Must be called first
db.query("SELECT * FROM users")  # CoE
```

**Strength**: Strong
**Refactoring**: Use initialization, state machines, or guards

#### Connascence of Timing (CoTi)
The timing of execution of multiple components is important.

**Example (JavaScript):**
```javascript
// Race condition example
let sharedCounter = 0;

async function incrementCounter() {
    const current = sharedCounter;  // Read
    await someAsyncOperation();     // Delay
    sharedCounter = current + 1;    // Write - CoTi with other calls
}
```

**Strength**: Very Strong
**Refactoring**: Use locks, immutable data, or message passing

#### Connascence of Values (CoV)
Multiple components must have certain values at the same time.

**Example (Java):**
```java
class BankAccount {
    private double balance;
    private double availableCredit;
    
    // These values must be consistent
    public void withdraw(double amount) {
        if (balance + availableCredit >= amount) {  // CoV
            balance -= amount;
            // availableCredit remains the same
        }
    }
}
```

**Strength**: Strong
**Refactoring**: Encapsulate related values, use invariants

#### Connascence of Identity (CoI)
Multiple components must reference the same entity.

**Example (Python):**
```python
# Singleton pattern creates CoI
class DatabaseManager:
    _instance = None
    
    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

# All components must use the same instance
db1 = DatabaseManager()  # CoI
db2 = DatabaseManager()  # Same instance as db1
```

**Strength**: Very Strong
**Refactoring**: Dependency injection, eliminate shared state

## Implementation in Code Context Graph

### Detection Algorithms

```rust
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnascenceInstance {
    pub id: String,
    pub connascence_type: ConnascenceType,
    pub strength: f32,
    pub locality: f32,
    pub degree: usize,
    pub entities: Vec<String>,
    pub description: String,
    pub file_locations: Vec<FileLocation>,
    pub refactoring_suggestions: Vec<RefactoringSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnascenceType {
    // Static
    Name,
    Type,
    Meaning,
    Position,
    Algorithm,
    // Dynamic
    Execution,
    Timing,
    Values,
    Identity,
}

pub struct ConnascenceDetector {
    language: Language,
    config: ConnascenceConfig,
}

impl ConnascenceDetector {
    pub fn analyze(&self, ast: &SimplifiedAST) -> Result<Vec<ConnascenceInstance>> {
        let mut instances = Vec::new();
        
        // Detect static connascence
        instances.extend(self.detect_name_connascence(ast)?);
        instances.extend(self.detect_type_connascence(ast)?);
        instances.extend(self.detect_meaning_connascence(ast)?);
        instances.extend(self.detect_position_connascence(ast)?);
        instances.extend(self.detect_algorithm_connascence(ast)?);
        
        // Detect dynamic connascence (requires flow analysis)
        instances.extend(self.detect_execution_connascence(ast)?);
        instances.extend(self.detect_values_connascence(ast)?);
        instances.extend(self.detect_identity_connascence(ast)?);
        
        // Calculate metrics for each instance
        for instance in &mut instances {
            instance.strength = self.calculate_strength(&instance);
            instance.locality = self.calculate_locality(&instance);
            instance.refactoring_suggestions = self.suggest_refactorings(&instance);
        }
        
        Ok(instances)
    }
}
```

### Static Connascence Detection

#### Name Connascence Detection

```rust
impl ConnascenceDetector {
    fn detect_name_connascence(&self, ast: &SimplifiedAST) -> Result<Vec<ConnascenceInstance>> {
        let mut instances = Vec::new();
        let mut name_usage = HashMap::<String, Vec<NameUsage>>::new();
        
        // Collect all name usages
        self.collect_name_usages(ast, &mut name_usage);
        
        // Find connascence instances
        for (name, usages) in name_usage {
            if usages.len() > 1 {
                let strength = self.calculate_name_strength(&usages);
                let locality = self.calculate_name_locality(&usages);
                
                if strength >= self.config.strength_threshold {
                    instances.push(ConnascenceInstance {
                        id: format!("con_name_{}", uuid::Uuid::new_v4()),
                        connascence_type: ConnascenceType::Name,
                        strength,
                        locality,
                        degree: usages.len(),
                        entities: usages.iter().map(|u| u.entity_id.clone()).collect(),
                        description: format!("Name '{}' used in {} locations", name, usages.len()),
                        file_locations: usages.iter().map(|u| u.location.clone()).collect(),
                        refactoring_suggestions: vec![],
                    });
                }
            }
        }
        
        Ok(instances)
    }
    
    fn collect_name_usages(&self, node: &ASTNode, usage_map: &mut HashMap<String, Vec<NameUsage>>) {
        match &node.node_type {
            NodeType::Function | NodeType::Class => {
                if let Some(name) = &node.name {
                    let usage = NameUsage {
                        name: name.clone(),
                        entity_id: node.id.clone(),
                        usage_type: NameUsageType::Definition,
                        location: FileLocation::from_range(&node.range),
                    };
                    usage_map.entry(name.clone()).or_default().push(usage);
                }
            }
            NodeType::Call => {
                // Extract function name from call
                if let Some(name) = self.extract_call_name(node) {
                    let usage = NameUsage {
                        name: name.clone(),
                        entity_id: node.id.clone(),
                        usage_type: NameUsageType::Reference,
                        location: FileLocation::from_range(&node.range),
                    };
                    usage_map.entry(name).or_default().push(usage);
                }
            }
            _ => {}
        }
        
        // Recursively process children
        for child in &node.children {
            self.collect_name_usages(child, usage_map);
        }
    }
}
```

#### Type Connascence Detection

```rust
impl ConnascenceDetector {
    fn detect_type_connascence(&self, ast: &SimplifiedAST) -> Result<Vec<ConnascenceInstance>> {
        let mut instances = Vec::new();
        let mut type_usage = HashMap::<String, Vec<TypeUsage>>::new();
        
        // Collect type usages
        self.collect_type_usages(ast, &mut type_usage);
        
        // Analyze type relationships
        for (type_name, usages) in type_usage {
            if usages.len() > 1 {
                let connascence = self.analyze_type_relationships(&usages);
                if let Some(instance) = connascence {
                    instances.push(instance);
                }
            }
        }
        
        Ok(instances)
    }
    
    fn collect_type_usages(&self, node: &ASTNode, usage_map: &mut HashMap<String, Vec<TypeUsage>>) {
        // Extract type annotations, parameter types, return types
        match &node.node_type {
            NodeType::Function => {
                // Check parameter types
                if let Some(params) = node.metadata.get("parameters") {
                    for param in params.as_array().unwrap_or(&vec![]) {
                        if let Some(type_name) = param.get("type").and_then(|t| t.as_str()) {
                            let usage = TypeUsage {
                                type_name: type_name.to_string(),
                                entity_id: node.id.clone(),
                                usage_context: TypeUsageContext::Parameter,
                                location: FileLocation::from_range(&node.range),
                            };
                            usage_map.entry(type_name.to_string()).or_default().push(usage);
                        }
                    }
                }
                
                // Check return type
                if let Some(return_type) = node.metadata.get("return_type") {
                    if let Some(type_name) = return_type.as_str() {
                        let usage = TypeUsage {
                            type_name: type_name.to_string(),
                            entity_id: node.id.clone(),
                            usage_context: TypeUsageContext::ReturnType,
                            location: FileLocation::from_range(&node.range),
                        };
                        usage_map.entry(type_name.to_string()).or_default().push(usage);
                    }
                }
            }
            _ => {}
        }
        
        // Recursively process children
        for child in &node.children {
            self.collect_type_usages(child, usage_map);
        }
    }
}
```

### Dynamic Connascence Detection

#### Execution Connascence Detection

```rust
impl ConnascenceDetector {
    fn detect_execution_connascence(&self, ast: &SimplifiedAST) -> Result<Vec<ConnascenceInstance>> {
        let mut instances = Vec::new();
        
        // Build control flow graph
        let cfg = self.build_control_flow_graph(ast)?;
        
        // Analyze method call sequences
        let call_sequences = self.extract_call_sequences(&cfg);
        
        for sequence in call_sequences {
            if self.has_execution_dependency(&sequence) {
                let instance = ConnascenceInstance {
                    id: format!("con_exec_{}", uuid::Uuid::new_v4()),
                    connascence_type: ConnascenceType::Execution,
                    strength: self.calculate_execution_strength(&sequence),
                    locality: self.calculate_execution_locality(&sequence),
                    degree: sequence.calls.len(),
                    entities: sequence.calls.iter().map(|c| c.entity_id.clone()).collect(),
                    description: format!("Execution order dependency in {} calls", sequence.calls.len()),
                    file_locations: sequence.calls.iter().map(|c| c.location.clone()).collect(),
                    refactoring_suggestions: self.suggest_execution_refactorings(&sequence),
                };
                instances.push(instance);
            }
        }
        
        Ok(instances)
    }
    
    fn has_execution_dependency(&self, sequence: &CallSequence) -> bool {
        // Check if calls modify shared state or have side effects
        for i in 0..sequence.calls.len() - 1 {
            let current = &sequence.calls[i];
            let next = &sequence.calls[i + 1];
            
            if self.modifies_state_used_by(current, next) {
                return true;
            }
        }
        
        false
    }
    
    fn modifies_state_used_by(&self, call1: &FunctionCall, call2: &FunctionCall) -> bool {
        // Analyze if call1 modifies state that call2 depends on
        let call1_effects = self.analyze_side_effects(call1);
        let call2_dependencies = self.analyze_dependencies(call2);
        
        !call1_effects.intersection(&call2_dependencies).is_empty()
    }
}
```

### Strength Calculation

```rust
impl ConnascenceDetector {
    fn calculate_strength(&self, instance: &ConnascenceInstance) -> f32 {
        let base_strength = match instance.connascence_type {
            ConnascenceType::Name => 0.1,
            ConnascenceType::Type => 0.2,
            ConnascenceType::Meaning => 0.4,
            ConnascenceType::Position => 0.6,
            ConnascenceType::Algorithm => 0.8,
            ConnascenceType::Execution => 0.9,
            ConnascenceType::Timing => 0.95,
            ConnascenceType::Values => 0.85,
            ConnascenceType::Identity => 0.9,
        };
        
        // Adjust based on degree (more elements = stronger)
        let degree_factor = 1.0 + (instance.degree as f32 - 2.0) * 0.1;
        
        // Adjust based on locality (distant elements = stronger)
        let locality_factor = 1.0 + (1.0 - instance.locality) * 0.5;
        
        (base_strength * degree_factor * locality_factor).min(1.0)
    }
    
    fn calculate_locality(&self, instance: &ConnascenceInstance) -> f32 {
        if instance.file_locations.len() < 2 {
            return 1.0; // Same location
        }
        
        let mut total_distance = 0.0;
        let mut comparisons = 0;
        
        for i in 0..instance.file_locations.len() {
            for j in i + 1..instance.file_locations.len() {
                let loc1 = &instance.file_locations[i];
                let loc2 = &instance.file_locations[j];
                
                total_distance += self.calculate_distance(loc1, loc2);
                comparisons += 1;
            }
        }
        
        if comparisons == 0 {
            return 1.0;
        }
        
        let avg_distance = total_distance / comparisons as f32;
        
        // Convert distance to locality (closer = higher locality)
        1.0 / (1.0 + avg_distance)
    }
    
    fn calculate_distance(&self, loc1: &FileLocation, loc2: &FileLocation) -> f32 {
        if loc1.file_path == loc2.file_path {
            // Same file - distance based on line difference
            (loc1.line as f32 - loc2.line as f32).abs() / 1000.0
        } else {
            // Different files - distance based on path similarity
            let path1_parts: Vec<&str> = loc1.file_path.split('/').collect();
            let path2_parts: Vec<&str> = loc2.file_path.split('/').collect();
            
            let common_prefix = path1_parts
                .iter()
                .zip(path2_parts.iter())
                .take_while(|(a, b)| a == b)
                .count();
            
            let total_parts = path1_parts.len() + path2_parts.len();
            let different_parts = total_parts - 2 * common_prefix;
            
            different_parts as f32
        }
    }
}
```

### Refactoring Suggestions

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    pub pattern: RefactoringPattern,
    pub description: String,
    pub estimated_effort: EffortLevel,
    pub benefits: Vec<String>,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringPattern {
    ExtractInterface,
    ExtractClass,
    IntroduceParameter,
    ReplaceWithBuilder,
    IntroduceStateObject,
    ExtractMethod,
    MoveMethod,
    IntroduceGuard,
    ReplaceWithStrategy,
    IntroduceFactory,
    ReplaceWithEnum,
}

impl ConnascenceDetector {
    fn suggest_refactorings(&self, instance: &ConnascenceInstance) -> Vec<RefactoringSuggestion> {
        match instance.connascence_type {
            ConnascenceType::Name => vec![
                RefactoringSuggestion {
                    pattern: RefactoringPattern::ExtractInterface,
                    description: "Extract interface to reduce name dependencies".to_string(),
                    estimated_effort: EffortLevel::Low,
                    benefits: vec![
                        "Reduces coupling".to_string(),
                        "Improves testability".to_string(),
                    ],
                    prerequisites: vec![],
                }
            ],
            
            ConnascenceType::Position => vec![
                RefactoringSuggestion {
                    pattern: RefactoringPattern::IntroduceParameter,
                    description: "Replace positional parameters with named parameters or parameter object".to_string(),
                    estimated_effort: EffortLevel::Medium,
                    benefits: vec![
                        "Eliminates parameter order dependency".to_string(),
                        "Improves code readability".to_string(),
                        "Reduces errors".to_string(),
                    ],
                    prerequisites: vec![
                        "Language supports named parameters".to_string(),
                    ],
                },
                RefactoringSuggestion {
                    pattern: RefactoringPattern::ReplaceWithBuilder,
                    description: "Use Builder pattern for complex parameter lists".to_string(),
                    estimated_effort: EffortLevel::High,
                    benefits: vec![
                        "Eliminates parameter order".to_string(),
                        "Allows optional parameters".to_string(),
                        "Improves API usability".to_string(),
                    ],
                    prerequisites: vec![
                        "Complex parameter list (>3 parameters)".to_string(),
                    ],
                }
            ],
            
            ConnascenceType::Execution => vec![
                RefactoringSuggestion {
                    pattern: RefactoringPattern::IntroduceGuard,
                    description: "Add precondition checks to enforce execution order".to_string(),
                    estimated_effort: EffortLevel::Low,
                    benefits: vec![
                        "Makes execution order explicit".to_string(),
                        "Fails fast on incorrect usage".to_string(),
                    ],
                    prerequisites: vec![],
                },
                RefactoringSuggestion {
                    pattern: RefactoringPattern::IntroduceStateObject,
                    description: "Use state machine to manage execution order".to_string(),
                    estimated_effort: EffortLevel::High,
                    benefits: vec![
                        "Enforces valid state transitions".to_string(),
                        "Makes state explicit".to_string(),
                        "Reduces execution connascence".to_string(),
                    ],
                    prerequisites: vec![
                        "Complex state management".to_string(),
                    ],
                }
            ],
            
            _ => vec![],
        }
    }
}
```

## Quality Metrics

### Connascence Scoring

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnascenceMetrics {
    pub total_instances: usize,
    pub by_type: HashMap<ConnascenceType, usize>,
    pub by_strength: StrengthDistribution,
    pub by_locality: LocalityDistribution,
    pub hotspots: Vec<ConnascenceHotspot>,
    pub overall_score: f32,
    pub quality_grade: QualityGrade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnascenceHotspot {
    pub entity_id: String,
    pub entity_name: String,
    pub connascence_count: usize,
    pub avg_strength: f32,
    pub risk_level: RiskLevel,
}

impl ConnascenceAnalyzer {
    pub fn calculate_metrics(&self, instances: &[ConnascenceInstance]) -> ConnascenceMetrics {
        let total_instances = instances.len();
        
        // Group by type
        let mut by_type = HashMap::new();
        for instance in instances {
            *by_type.entry(instance.connascence_type.clone()).or_insert(0) += 1;
        }
        
        // Calculate strength distribution
        let by_strength = self.calculate_strength_distribution(instances);
        
        // Calculate locality distribution
        let by_locality = self.calculate_locality_distribution(instances);
        
        // Find hotspots
        let hotspots = self.find_hotspots(instances);
        
        // Calculate overall score
        let overall_score = self.calculate_overall_score(instances);
        
        // Determine quality grade
        let quality_grade = self.determine_quality_grade(overall_score);
        
        ConnascenceMetrics {
            total_instances,
            by_type,
            by_strength,
            by_locality,
            hotspots,
            overall_score,
            quality_grade,
        }
    }
    
    fn calculate_overall_score(&self, instances: &[ConnascenceInstance]) -> f32 {
        if instances.is_empty() {
            return 1.0;
        }
        
        let total_weight = instances.len() as f32;
        let weighted_strength_sum: f32 = instances
            .iter()
            .map(|i| {
                let locality_penalty = 1.0 - i.locality;
                let degree_penalty = (i.degree as f32).log2() / 10.0;
                i.strength * (1.0 + locality_penalty + degree_penalty)
            })
            .sum();
        
        1.0 - (weighted_strength_sum / total_weight).min(1.0)
    }
}
```

## Configuration

### Connascence Detection Configuration

```toml
[connascence]
enabled = true
detect_static = true
detect_dynamic = true
strength_threshold = 0.7
locality_threshold = 0.5
degree_threshold = 5
auto_suggest_refactoring = true
export_metrics = true

[connascence.types]
name = { enabled = true, weight = 1.0 }
type = { enabled = true, weight = 1.2 }
meaning = { enabled = true, weight = 1.4 }
position = { enabled = true, weight = 1.6 }
algorithm = { enabled = true, weight = 1.8 }
execution = { enabled = true, weight = 2.0 }
timing = { enabled = false, weight = 2.2 }  # Experimental
values = { enabled = true, weight = 2.4 }
identity = { enabled = true, weight = 2.6 }

[connascence.reporting]
min_strength = 0.5
include_suggestions = true
group_by_file = true
export_format = "json"  # json, csv, html
```

## Usage Examples

### CLI Usage

```bash
# Analyze connascence in a project
ccg connascence analyze ./src --min-strength 0.7

# Generate connascence report
ccg connascence report ./src --format html --output connascence-report.html

# Show hotspots
ccg connascence hotspots ./src --top 10

# Suggest refactorings
ccg connascence refactor ./src/payment.py --interactive
```

### API Usage

```python
import requests

# Analyze connascence
response = requests.post("http://localhost:8080/api/v1/quality/connascence", json={
    "target": {
        "type": "module",
        "path": "src/services/payment"
    },
    "connascence_types": ["Execution", "Position"],
    "min_strength": 0.7,
    "include_suggestions": true
})

connascence_data = response.json()
print(f"Found {len(connascence_data['analysis']['connascence_instances'])} instances")

for instance in connascence_data['analysis']['connascence_instances']:
    print(f"- {instance['type']}: {instance['description']}")
    print(f"  Strength: {instance['strength']:.2f}")
    print(f"  Entities: {', '.join(instance['entities'])}")
    
    if instance.get('refactoring_suggestions'):
        print("  Refactoring suggestions:")
        for suggestion in instance['refactoring_suggestions']:
            print(f"    * {suggestion['description']}")
```

## Best Practices

### Connascence Guidelines

1. **Minimize Dynamic Connascence**: Dynamic forms are harder to detect and refactor
2. **Keep Strong Connascence Local**: High-strength coupling should be within the same module
3. **Reduce Degree**: Fewer components should share the same connascence
4. **Refactor Systematically**: Address highest-strength, lowest-locality instances first

### Refactoring Priority

```
Priority = Strength × (1 - Locality) × log(Degree)
```

Higher priority indicates more urgent refactoring need.

### Integration with Development Workflow

```yaml
# .github/workflows/connascence-check.yml
name: Connascence Analysis
on: [push, pull_request]

jobs:
  connascence:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Analyze Connascence
        run: |
          ccg connascence analyze ./src --min-strength 0.8 --fail-on-violations
          ccg connascence report ./src --format json --output connascence.json
      - name: Upload Report
        uses: actions/upload-artifact@v2
        with:
          name: connascence-report
          path: connascence.json
```

This comprehensive connascence analysis helps maintain code quality by identifying and suggesting refactorings for coupling issues that traditional metrics might miss.