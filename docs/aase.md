# AASE Context Engineering Guide

## Overview

AASE (Automatización Asistida por IA / AI-Assisted Automation) is a methodology for creating rich, contextual artifacts that enable effective human-AI collaboration in software development. Code Context Graph implements a comprehensive AASE system for context generation, management, and evolution.

## AASE Principles

### Core Concepts

1. **Context Over Configuration**: Rich contextual information reduces the need for detailed configuration
2. **Evolutionary Intelligence**: Artifacts improve over time through feedback loops
3. **Human-in-the-Loop**: Critical decisions involve human oversight and validation
4. **Traceability**: Every artifact has a complete audit trail of its creation and evolution
5. **Convention-Driven**: Standardized naming and structure reduce friction

### Context Chain

AASE artifacts form hierarchical chains from high-level domain knowledge to specific implementation details:

```
Domain Context → Model → Use Case → Prompt → Generated Code
     ↓             ↓        ↓         ↓          ↓
   CTX-*         MDL-*    UCS-*     PRM-*    Generated
```

## Artifact Types

### Context Artifacts (CTX-*)

Domain-level context that describes business rules, constraints, and conceptual models.

**Structure:**
```toml
[artifact]
id = "CTX-payment-domain-v3"
type = "Context"
version = 3
created_at = "2025-08-03T10:00:00Z"
created_by = "domain-expert"
parent_version = "CTX-payment-domain-v2"

[domain]
name = "Payment Processing"
description = "Secure payment processing with multiple gateway support"
bounded_context = "financial-transactions"

[business_rules]
rules = [
    "All payments must be validated before processing",
    "Failed payments should be retried up to 3 times",
    "Refunds must be processed within 24 hours",
    "PCI compliance is mandatory for all card transactions"
]

[quality_attributes]
security = { priority = "critical", requirements = ["PCI DSS", "encryption", "audit-trail"] }
performance = { priority = "high", requirements = ["<2s response", "99.9% uptime"] }
scalability = { priority = "medium", requirements = ["1000 TPS", "horizontal scaling"] }

[key_concepts]
payment = "Financial transaction between customer and merchant"
gateway = "External service for processing card transactions"
refund = "Reversal of a previously completed payment"
fraud_check = "Automated verification of transaction legitimacy"

[constraints]
technical = [
    "Must integrate with existing user management system",
    "Database transactions required for payment operations",
    "Async processing for non-critical operations"
]
business = [
    "Maximum transaction amount: $10,000",
    "Supported currencies: USD, EUR, GBP",
    "Business hours: 24/7 for payments, 9-5 for support"
]

[relationships]
depends_on = ["CTX-user-domain-v2", "CTX-notification-domain-v1"]
influences = ["MDL-payment-entities-v1", "UCS-process-payment-v2"]

[metrics]
quality_score = 0.92
completeness = 0.89
consistency = 0.95
traceability = 1.0
```

### Model Artifacts (MDL-*)

Technical models that define data structures, interfaces, and system architecture.

**Structure:**
```toml
[artifact]
id = "MDL-payment-entities-v1"
type = "Model"
version = 1
created_at = "2025-08-03T10:15:00Z"
source_context = "CTX-payment-domain-v3"

[entities.Payment]
description = "Core payment entity representing a financial transaction"
attributes = [
    { name = "id", type = "UUID", required = true, description = "Unique payment identifier" },
    { name = "amount", type = "Decimal", required = true, constraints = ["positive", "max_precision=2"] },
    { name = "currency", type = "Currency", required = true, constraints = ["ISO_4217"] },
    { name = "status", type = "PaymentStatus", required = true },
    { name = "created_at", type = "DateTime", required = true },
    { name = "gateway_id", type = "String", required = false },
]
relationships = [
    { name = "customer", type = "User", cardinality = "many-to-one" },
    { name = "refunds", type = "Refund", cardinality = "one-to-many" },
]
business_rules = [
    "Amount must be positive",
    "Currency must match merchant settings",
    "Status transitions must follow state machine"
]

[entities.PaymentGateway]
description = "External payment processing service"
attributes = [
    { name = "id", type = "String", required = true },
    { name = "name", type = "String", required = true },
    { name = "endpoint", type = "URL", required = true },
    { name = "credentials", type = "EncryptedString", required = true },
]
capabilities = ["card_processing", "refunds", "fraud_detection"]

[state_machines.PaymentStatus]
states = ["pending", "processing", "completed", "failed", "refunded"]
transitions = [
    { from = "pending", to = "processing", trigger = "gateway_response" },
    { from = "processing", to = "completed", trigger = "success" },
    { from = "processing", to = "failed", trigger = "error" },
    { from = "completed", to = "refunded", trigger = "refund_request" },
]

[interfaces.PaymentProcessor]
methods = [
    {
        name = "process_payment",
        parameters = ["payment_request: PaymentRequest"],
        returns = "Result<PaymentResponse, PaymentError>",
        async = true
    },
    {
        name = "refund_payment",
        parameters = ["payment_id: UUID", "amount: Option<Decimal>"],
        returns = "Result<RefundResponse, RefundError>",
        async = true
    }
]
```

### Use Case Artifacts (UCS-*)

Specific scenarios and workflows that combine domain context with technical implementation.

**Structure:**
```toml
[artifact]
id = "UCS-process-payment-v2"
type = "UseCase"
version = 2
source_context = "CTX-payment-domain-v3"
source_model = "MDL-payment-entities-v1"

[use_case]
name = "Process Customer Payment"
description = "End-to-end payment processing including validation, gateway integration, and result handling"
actor = "Customer"
trigger = "Checkout completion"

[preconditions]
conditions = [
    "Customer is authenticated",
    "Shopping cart contains items",
    "Payment method is valid",
    "Gateway is available"
]

[flow.main]
steps = [
    {
        step = 1,
        action = "Validate payment request",
        actor = "System",
        details = "Check amount, currency, payment method validity"
    },
    {
        step = 2,
        action = "Create payment record",
        actor = "System",
        details = "Insert payment with 'pending' status"
    },
    {
        step = 3,
        action = "Submit to gateway",
        actor = "System",
        details = "Send payment request to configured gateway"
    },
    {
        step = 4,
        action = "Update payment status",
        actor = "System",
        details = "Set status based on gateway response"
    },
    {
        step = 5,
        action = "Send notification",
        actor = "System",
        details = "Notify customer of payment result"
    }
]

[flow.exceptions]
gateway_timeout = {
    description = "Gateway doesn't respond within timeout",
    handling = "Retry up to 3 times, then mark as failed",
    compensation = "Release reserved inventory"
}
insufficient_funds = {
    description = "Customer's payment method declined",
    handling = "Mark payment as failed, notify customer",
    compensation = "None required"
}
fraud_detected = {
    description = "Fraud detection system flags transaction",
    handling = "Hold payment for manual review",
    compensation = "Reserve inventory for 24 hours"
}

[postconditions.success]
conditions = [
    "Payment status is 'completed'",
    "Customer charged successfully",
    "Inventory updated",
    "Receipt sent to customer"
]

[postconditions.failure]
conditions = [
    "Payment status is 'failed'",
    "No charge applied",
    "Inventory released",
    "Error notification sent"
]

[quality_requirements]
performance = { response_time = "< 2 seconds", throughput = "1000 TPS" }
reliability = { success_rate = "> 99.5%", retry_limit = 3 }
security = { encryption = "AES-256", audit_logging = true }
```

### Prompt Artifacts (PRM-*)

Specific prompts for code generation, documentation, or other AI-assisted tasks.

**Structure:**
```toml
[artifact]
id = "PRM-payment-service-impl-v1"
type = "Prompt"
version = 1
source_use_case = "UCS-process-payment-v2"
target_language = "rust"

[prompt]
template = """
# Payment Service Implementation

Generate a Rust implementation for the payment processing service based on the following context:

## Domain Context
{{context.domain_description}}

## Key Requirements
{{#each context.business_rules}}
- {{this}}
{{/each}}

## Entities
{{#each model.entities}}
### {{@key}}
{{this.description}}

Attributes:
{{#each this.attributes}}
- {{name}}: {{type}}{{#if required}} (required){{/if}}
{{/each}}
{{/each}}

## Use Case Flow
{{#each use_case.flow.main}}
{{step}}. {{action}} - {{details}}
{{/each}}

## Implementation Guidelines
- Use async/await for external calls
- Implement proper error handling with custom error types
- Add comprehensive logging for audit trail
- Include input validation
- Follow Rust best practices and idioms

## Code Structure
```rust
// Define the service struct and implementation
pub struct PaymentService {
    // Add necessary fields
}

impl PaymentService {
    // Implement the main process_payment method
    pub async fn process_payment(&self, request: PaymentRequest) -> Result<PaymentResponse, PaymentError> {
        // Implementation here
    }
}
```

Please generate a complete, production-ready implementation.
"""

[variables]
context = { source = "CTX-payment-domain-v3" }
model = { source = "MDL-payment-entities-v1" }
use_case = { source = "UCS-process-payment-v2" }

[generation_config]
temperature = 0.3
max_tokens = 2000
stop_sequences = ["---END---"]
model = "claude-3-sonnet"

[validation_rules]
compilation_check = true
lint_check = true
test_coverage_min = 0.8
documentation_required = true
```

## Implementation Architecture

### Context Store

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AASEArtifact {
    pub id: String,
    pub artifact_type: ArtifactType,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub parent_version: Option<String>,
    pub content: serde_json::Value,
    pub metadata: ArtifactMetadata,
    pub relationships: Vec<ArtifactRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Context,
    Model,
    UseCase,
    Prompt,
    Specification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub quality_score: f32,
    pub completeness: f32,
    pub consistency: f32,
    pub traceability: f32,
    pub usage_count: u32,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Vec<String>,
}

pub struct ContextStore {
    storage: Box<dyn ArtifactStorage>,
    validators: HashMap<ArtifactType, Box<dyn ArtifactValidator>>,
    generators: HashMap<ArtifactType, Box<dyn ArtifactGenerator>>,
}

impl ContextStore {
    pub async fn create_artifact(&mut self, artifact: AASEArtifact) -> Result<String, AASEError> {
        // Validate artifact structure
        if let Some(validator) = self.validators.get(&artifact.artifact_type) {
            validator.validate(&artifact)?;
        }
        
        // Check dependencies
        self.validate_dependencies(&artifact).await?;
        
        // Store artifact
        let artifact_id = self.storage.store(artifact).await?;
        
        // Update dependent artifacts
        self.propagate_changes(&artifact_id).await?;
        
        Ok(artifact_id)
    }
    
    pub async fn get_context_chain(&self, artifact_id: &str) -> Result<Vec<AASEArtifact>, AASEError> {
        let mut chain = Vec::new();
        let mut current_id = Some(artifact_id.to_string());
        
        while let Some(id) = current_id {
            let artifact = self.storage.get(&id).await?;
            
            // Find source artifacts
            let sources = self.find_source_artifacts(&artifact).await?;
            chain.extend(sources);
            
            current_id = artifact.parent_version;
        }
        
        // Sort by dependency order
        self.sort_by_dependency_order(&mut chain);
        
        Ok(chain)
    }
}
```

### Context Generation

```rust
pub struct ContextGenerator {
    llm_client: Box<dyn LLMClient>,
    template_engine: TemplateEngine,
    quality_analyzer: QualityAnalyzer,
}

impl ContextGenerator {
    pub async fn generate_context(
        &self,
        domain: &str,
        code_entities: &[CodeNode],
        existing_context: Option<&AASEArtifact>,
    ) -> Result<AASEArtifact, AASEError> {
        
        // Analyze code entities for domain concepts
        let domain_concepts = self.extract_domain_concepts(code_entities)?;
        
        // Build generation prompt
        let prompt = self.build_context_prompt(domain, &domain_concepts, existing_context)?;
        
        // Generate context using LLM
        let generated_content = self.llm_client.generate(&prompt).await?;
        
        // Parse and validate generated content
        let context_content = self.parse_context_content(&generated_content)?;
        
        // Calculate quality metrics
        let quality_metrics = self.quality_analyzer.analyze_context(&context_content)?;
        
        // Create artifact
        let artifact = AASEArtifact {
            id: format!("CTX-{}-v1", domain.to_lowercase().replace(' ', "-")),
            artifact_type: ArtifactType::Context,
            version: 1,
            created_at: chrono::Utc::now(),
            created_by: "system-generator".to_string(),
            parent_version: existing_context.map(|c| c.id.clone()),
            content: context_content,
            metadata: ArtifactMetadata {
                quality_score: quality_metrics.overall_score,
                completeness: quality_metrics.completeness,
                consistency: quality_metrics.consistency,
                traceability: quality_metrics.traceability,
                usage_count: 0,
                last_used: None,
                tags: vec![domain.to_lowercase()],
            },
            relationships: self.extract_relationships(code_entities),
        };
        
        Ok(artifact)
    }
    
    fn extract_domain_concepts(&self, entities: &[CodeNode]) -> Result<Vec<DomainConcept>, AASEError> {
        let mut concepts = Vec::new();
        
        for entity in entities {
            match entity.node_type {
                CodeNodeType::Class => {
                    concepts.push(DomainConcept {
                        name: entity.name.clone(),
                        concept_type: ConceptType::Entity,
                        description: self.infer_description(entity)?,
                        attributes: self.extract_attributes(entity)?,
                        relationships: self.extract_entity_relationships(entity)?,
                    });
                }
                CodeNodeType::Function => {
                    concepts.push(DomainConcept {
                        name: entity.name.clone(),
                        concept_type: ConceptType::Process,
                        description: self.infer_description(entity)?,
                        attributes: vec![],
                        relationships: vec![],
                    });
                }
                _ => {}
            }
        }
        
        Ok(concepts)
    }
}
```

### Template System

```rust
use handlebars::Handlebars;

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        
        // Register built-in templates
        handlebars.register_template_string("context", include_str!("templates/context.hbs")).unwrap();
        handlebars.register_template_string("model", include_str!("templates/model.hbs")).unwrap();
        handlebars.register_template_string("use_case", include_str!("templates/use_case.hbs")).unwrap();
        handlebars.register_template_string("prompt", include_str!("templates/prompt.hbs")).unwrap();
        
        // Register helpers
        handlebars.register_helper("pluralize", Box::new(pluralize_helper));
        handlebars.register_helper("camel_case", Box::new(camel_case_helper));
        handlebars.register_helper("snake_case", Box::new(snake_case_helper));
        
        Self { handlebars }
    }
    
    pub fn render_template(
        &self,
        template_name: &str,
        data: &serde_json::Value,
    ) -> Result<String, TemplateError> {
        self.handlebars.render(template_name, data)
            .map_err(TemplateError::from)
    }
}

// Helper functions
fn pluralize_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let word = h.param(0).unwrap().value().as_str().unwrap();
    let pluralized = match word {
        s if s.ends_with("y") => format!("{}ies", &s[..s.len()-1]),
        s if s.ends_with("s") || s.ends_with("sh") || s.ends_with("ch") => format!("{}es", s),
        s => format!("{}s", s),
    };
    out.write(&pluralized)?;
    Ok(())
}
```

### Context Evolution

```rust
pub struct ContextEvolutionEngine {
    change_detector: ChangeDetector,
    impact_analyzer: ImpactAnalyzer,
    update_proposer: UpdateProposer,
    validation_engine: ValidationEngine,
}

impl ContextEvolutionEngine {
    pub async fn evolve_contexts(
        &self,
        code_changes: &[CodeChange],
    ) -> Result<Vec<ContextUpdate>, AASEError> {
        
        // Detect which contexts are affected
        let affected_contexts = self.change_detector.find_affected_contexts(code_changes).await?;
        
        let mut updates = Vec::new();
        
        for context in affected_contexts {
            // Analyze impact of changes
            let impact = self.impact_analyzer.analyze(&context, code_changes).await?;
            
            if impact.requires_update() {
                // Propose updates
                let proposed_update = self.update_proposer.propose_update(&context, &impact).await?;
                
                // Validate proposed update
                let validation_result = self.validation_engine.validate(&proposed_update).await?;
                
                if validation_result.is_valid() {
                    updates.push(ContextUpdate {
                        context_id: context.id.clone(),
                        update_type: impact.update_type(),
                        proposed_changes: proposed_update,
                        confidence: validation_result.confidence,
                        requires_human_review: validation_result.requires_human_review,
                    });
                }
            }
        }
        
        Ok(updates)
    }
}

#[derive(Debug, Clone)]
pub struct ContextUpdate {
    pub context_id: String,
    pub update_type: UpdateType,
    pub proposed_changes: ProposedUpdate,
    pub confidence: f32,
    pub requires_human_review: bool,
}

#[derive(Debug, Clone)]
pub enum UpdateType {
    ContentUpdate,
    StructuralChange,
    RelationshipChange,
    QualityImprovement,
    DeprecationWarning,
}
```

## Human-in-the-Loop Integration

### Review Workflow

```rust
pub struct HumanReviewWorkflow {
    review_queue: ReviewQueue,
    notification_system: NotificationSystem,
    feedback_collector: FeedbackCollector,
}

impl HumanReviewWorkflow {
    pub async fn submit_for_review(
        &self,
        update: ContextUpdate,
        reviewer: &str,
    ) -> Result<ReviewTask, AASEError> {
        
        let task = ReviewTask {
            id: Uuid::new_v4().to_string(),
            context_update: update,
            assigned_to: reviewer.to_string(),
            created_at: chrono::Utc::now(),
            status: ReviewStatus::Pending,
            metadata: ReviewMetadata::default(),
        };
        
        // Add to review queue
        self.review_queue.enqueue(&task).await?;
        
        // Notify reviewer
        self.notification_system.notify_reviewer(&task).await?;
        
        Ok(task)
    }
    
    pub async fn process_review_feedback(
        &self,
        task_id: &str,
        feedback: ReviewFeedback,
    ) -> Result<(), AASEError> {
        
        let mut task = self.review_queue.get_task(task_id).await?;
        
        match feedback.decision {
            ReviewDecision::Approved => {
                task.status = ReviewStatus::Approved;
                
                // Apply the context update
                self.apply_context_update(&task.context_update).await?;
                
                // Collect positive feedback for learning
                self.feedback_collector.record_approval(&task, &feedback).await?;
            }
            ReviewDecision::Rejected => {
                task.status = ReviewStatus::Rejected;
                
                // Collect negative feedback for learning
                self.feedback_collector.record_rejection(&task, &feedback).await?;
            }
            ReviewDecision::Modified => {
                task.status = ReviewStatus::Modified;
                
                // Apply modified update
                let modified_update = feedback.modified_update.unwrap();
                self.apply_context_update(&modified_update).await?;
                
                // Learn from the modifications
                self.feedback_collector.record_modification(&task, &feedback).await?;
            }
        }
        
        self.review_queue.update_task(&task).await?;
        
        Ok(())
    }
}
```

## Quality Assurance

### Context Validation

```rust
pub struct ContextValidator {
    schema_validator: SchemaValidator,
    consistency_checker: ConsistencyChecker,
    completeness_analyzer: CompletenessAnalyzer,
    traceability_verifier: TraceabilityVerifier,
}

impl ContextValidator {
    pub async fn validate_context(&self, context: &AASEArtifact) -> Result<ValidationResult, AASEError> {
        let mut issues = Vec::new();
        let mut metrics = ValidationMetrics::default();
        
        // Schema validation
        if let Err(schema_errors) = self.schema_validator.validate(&context.content) {
            issues.extend(schema_errors.into_iter().map(ValidationIssue::SchemaViolation));
        }
        
        // Consistency checking
        let consistency_result = self.consistency_checker.check(context).await?;
        metrics.consistency = consistency_result.score;
        issues.extend(consistency_result.issues.into_iter().map(ValidationIssue::ConsistencyError));
        
        // Completeness analysis
        let completeness_result = self.completeness_analyzer.analyze(context).await?;
        metrics.completeness = completeness_result.score;
        issues.extend(completeness_result.missing_elements.into_iter().map(ValidationIssue::MissingElement));
        
        // Traceability verification
        let traceability_result = self.traceability_verifier.verify(context).await?;
        metrics.traceability = traceability_result.score;
        issues.extend(traceability_result.broken_links.into_iter().map(ValidationIssue::BrokenTraceability));
        
        // Calculate overall quality score
        metrics.overall_score = (metrics.consistency + metrics.completeness + metrics.traceability) / 3.0;
        
        Ok(ValidationResult {
            is_valid: issues.is_empty(),
            quality_score: metrics.overall_score,
            metrics,
            issues,
        })
    }
}
```

## Configuration

### AASE Configuration

```toml
[aase]
enabled = true
context_path = "./context"
naming_convention = "strict"
auto_propagate = true
human_review_threshold = 0.8
artifact_versioning = true
context_chain_depth = 5
template_path = "./templates"

[aase.artifact_types]
context = { prefix = "CTX", enabled = true }
model = { prefix = "MDL", enabled = true }
use_case = { prefix = "UCS", enabled = true }
prompt = { prefix = "PRM", enabled = true }
specification = { prefix = "SPC", enabled = true }

[aase.generation]
llm_model = "claude-3-sonnet"
temperature = 0.3
max_tokens = 4000
enable_caching = true
cache_ttl_hours = 24

[aase.quality]
min_completeness = 0.7
min_consistency = 0.8
min_traceability = 0.9
auto_improve = true
improvement_threshold = 0.1

[aase.human_review]
enabled = true
required_for_structural_changes = true
required_for_low_confidence = true
notification_method = "email"
default_reviewer = "domain-expert@company.com"
review_timeout_hours = 48

[aase.evolution]
auto_update_enabled = true
change_detection_sensitivity = 0.5
impact_analysis_depth = 3
batch_updates = true
update_interval_minutes = 15
```

## Usage Examples

### CLI Usage

```bash
# Generate context for a domain
ccg aase generate-context --domain payment --analyze ./src/payment

# Create use case from context
ccg aase create-use-case --context CTX-payment-domain-v3 --scenario "process payment"

# Generate code from context chain
ccg aase generate-code --use-case UCS-process-payment-v2 --language rust

# Validate context artifacts
ccg aase validate --artifact CTX-payment-domain-v3

# Show context chain
ccg aase chain --artifact PRM-payment-service-impl-v1

# Export context for external use
ccg aase export --format json --output payment-context.json CTX-payment-domain-v3
```

### API Usage

```python
import requests

# Generate context artifact
response = requests.post("http://localhost:8080/api/v1/aase/generate", json={
    "domain": "user-management",
    "artifact_type": "Context",
    "base_entities": ["class_user_service", "func_authenticate_user"],
    "include_connascence": True
})

context_id = response.json()["artifact_id"]

# Create use case from context
response = requests.post("http://localhost:8080/api/v1/aase/artifacts", json={
    "type": "UseCase",
    "source_context": context_id,
    "scenario": "User Registration Flow",
    "actor": "New User"
})

use_case_id = response.json()["artifact_id"]

# Generate code from use case
response = requests.post("http://localhost:8080/api/v1/aase/generate-code", json={
    "use_case_id": use_case_id,
    "target_language": "rust",
    "style": "async-tokio"
})

generated_code = response.json()["code"]
print(generated_code)
```

### Integration with Development Workflow

```yaml
# .github/workflows/aase-update.yml
name: AASE Context Update
on:
  push:
    paths: ['src/**']

jobs:
  update-context:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Analyze Code Changes
        run: |
          ccg aase analyze-changes --since ${{ github.event.before }} --format json > changes.json
      
      - name: Update Contexts
        run: |
          ccg aase evolve-contexts --changes changes.json --auto-approve-threshold 0.9
      
      - name: Generate Review Tasks
        run: |
          ccg aase review-queue --format json > review-tasks.json
      
      - name: Create PR for Context Updates
        if: ${{ steps.update-contexts.outputs.has_updates == 'true' }}
        uses: peter-evans/create-pull-request@v3
        with:
          title: "AASE: Update context artifacts"
          body: "Automated context updates based on code changes"
          branch: "aase/context-updates"
```

## Best Practices

### Context Design Guidelines

1. **Start with Domain**: Begin with domain context before technical details
2. **Maintain Traceability**: Every artifact should trace back to business requirements
3. **Version Everything**: All artifacts should be versioned and auditable
4. **Human Oversight**: Critical decisions should involve human review
5. **Iterative Improvement**: Contexts should evolve based on feedback and usage

### Quality Metrics

- **Completeness**: Are all necessary elements present?
- **Consistency**: Are there contradictions within or between artifacts?
- **Traceability**: Can you trace from code back to business requirements?
- **Usability**: How effectively do the artifacts enable AI-assisted development?

### Integration Points

AASE integrates with all other Code Context Graph components:

- **Parser**: Code analysis feeds into context generation
- **Connascence**: Coupling analysis informs refactoring suggestions in contexts
- **CAS/Merkle**: Artifact versioning leverages the same storage system
- **API**: Contexts are queryable through the main API
- **File Watcher**: Code changes trigger context evolution

This comprehensive AASE system enables rich, contextual AI assistance throughout the software development lifecycle.