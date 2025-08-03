# API Reference

## Overview

The Code Context Graph API provides REST endpoints for querying the semantic graph, managing versions, analyzing code quality, and retrieving AASE context artifacts.

Base URL: `http://localhost:8080` (configurable)

## Authentication

Currently, the API uses API key authentication:

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:8080/api/v1/query
```

## Core Endpoints

### Graph Query API

#### POST /api/v1/query

Query the semantic graph with natural language or structured queries.

**Request Body:**
```json
{
  "question": "What functions call authenticateUser?",
  "max_hops": 3,
  "include_code": true,
  "include_quality_metrics": true,
  "version": "latest",
  "filters": {
    "languages": ["python", "javascript"],
    "file_patterns": ["src/**/*.py"],
    "exclude_tests": true
  }
}
```

**Response:**
```json
{
  "query_id": "q_abc123",
  "context": {
    "primary_entities": [
      {
        "id": "func_authenticate_user",
        "name": "authenticateUser",
        "type": "Function",
        "language": "Python",
        "file_path": "src/auth/service.py",
        "line_range": [45, 72],
        "source_snippet": "def authenticateUser(username, password):\n    # Authentication logic\n    return verify_credentials(username, password)",
        "metadata": {
          "complexity": 8,
          "parameters": ["username", "password"],
          "return_type": "bool"
        }
      }
    ],
    "relationships": [
      {
        "from": "func_login_endpoint",
        "to": "func_authenticate_user",
        "type": "calls",
        "file_path": "src/api/auth.py",
        "line": 23
      }
    ],
    "quality_metrics": {
      "cohesion": 0.85,
      "afferent_coupling": 3,
      "efferent_coupling": 2,
      "instability": 0.4,
      "connascence_score": 0.3
    },
    "connascence": [
      {
        "type": "Name",
        "strength": 0.6,
        "entities": ["authenticateUser", "authenticate_user"],
        "description": "Inconsistent naming convention"
      }
    ]
  },
  "related_entities": [
    "func_verify_credentials",
    "class_user_repository"
  ],
  "version_info": {
    "version_id": "v_def456",
    "merkle_root": "mr_789abc",
    "timestamp": "2025-08-03T10:30:00Z"
  }
}
```

#### GET /api/v1/graph/nodes/{node_id}

Retrieve detailed information about a specific node.

**Response:**
```json
{
  "node": {
    "id": "class_payment_service",
    "name": "PaymentService",
    "type": "Class",
    "language": "Python",
    "file_path": "src/services/payment.py",
    "line_range": [10, 150],
    "metadata": {
      "methods": ["process_payment", "validate_card", "send_receipt"],
      "inheritance": ["BaseService"],
      "interfaces": ["PaymentProcessor"]
    },
    "quality_metrics": {
      "lines_of_code": 140,
      "cyclomatic_complexity": 12,
      "maintainability_index": 68
    }
  },
  "relationships": {
    "incoming": [
      {
        "from": "class_checkout_controller",
        "type": "instantiates",
        "line": 45
      }
    ],
    "outgoing": [
      {
        "to": "class_payment_gateway",
        "type": "uses",
        "line": 67
      }
    ]
  }
}
```

### Version Management API

#### GET /api/v1/versions

List all versions of the codebase.

**Query Parameters:**
- `limit`: Number of versions to return (default: 50)
- `offset`: Pagination offset
- `author`: Filter by author
- `since`: ISO timestamp for filtering

**Response:**
```json
{
  "versions": [
    {
      "version_id": "v_abc123",
      "merkle_root": "mr_def456",
      "timestamp": "2025-08-03T10:30:00Z",
      "author": "developer@example.com",
      "change_summary": {
        "files_added": 2,
        "files_modified": 5,
        "files_deleted": 1,
        "entities_changed": 12
      },
      "quality_delta": {
        "cohesion": 0.02,
        "coupling": -0.05,
        "connascence_score": -0.1
      }
    }
  ],
  "pagination": {
    "total": 157,
    "has_next": true,
    "next_offset": 50
  }
}
```

#### POST /api/v1/versions/compare

Compare two versions of the codebase.

**Request Body:**
```json
{
  "from_version": "v_abc123",
  "to_version": "v_def456",
  "entity_filter": "payment",
  "include_quality_delta": true,
  "diff_type": "semantic"
}
```

**Response:**
```json
{
  "comparison": {
    "from_version": "v_abc123",
    "to_version": "v_def456",
    "changed_files": [
      {
        "path": "src/services/payment.py",
        "change_type": "modified",
        "lines_changed": 23
      }
    ],
    "affected_entities": [
      {
        "id": "func_process_payment",
        "change_type": "modified",
        "changes": ["signature", "implementation"]
      }
    ],
    "quality_delta": {
      "cohesion": 0.05,
      "coupling": -0.02,
      "connascence_changes": [
        {
          "type": "Type",
          "old_strength": 0.6,
          "new_strength": 0.4,
          "impact": "Reduced coupling through interface extraction"
        }
      ]
    }
  }
}
```

### Quality Analysis API

#### GET /api/v1/quality/overview

Get overall quality metrics for the codebase.

**Response:**
```json
{
  "overall_metrics": {
    "total_entities": 1250,
    "average_cohesion": 0.72,
    "average_coupling": 0.45,
    "maintainability_index": 65,
    "technical_debt_ratio": 0.15
  },
  "connascence_summary": {
    "total_instances": 89,
    "by_type": {
      "Name": 34,
      "Type": 28,
      "Execution": 15,
      "Position": 12
    },
    "high_strength_count": 23,
    "avg_strength": 0.42
  },
  "hotspots": [
    {
      "entity_id": "class_user_service",
      "issues": ["high_coupling", "low_cohesion"],
      "priority": "high"
    }
  ]
}
```

#### POST /api/v1/quality/connascence

Analyze connascence in specific modules or entities.

**Request Body:**
```json
{
  "target": {
    "type": "module",
    "path": "src/services/payment"
  },
  "connascence_types": ["Execution", "Timing"],
  "min_strength": 0.7,
  "include_suggestions": true
}
```

**Response:**
```json
{
  "analysis": {
    "target": "src/services/payment",
    "connascence_instances": [
      {
        "id": "conn_exec_1",
        "type": "Execution",
        "strength": 0.85,
        "entities": [
          "func_validate_card",
          "func_process_payment"
        ],
        "description": "validate_card must be called before process_payment",
        "location": {
          "file": "src/services/payment.py",
          "lines": [45, 67]
        },
        "impact": "High - payment failure if order violated"
      }
    ],
    "refactoring_suggestions": [
      {
        "connascence_id": "conn_exec_1",
        "suggestion": "Extract validation to separate service",
        "pattern": "Strategy Pattern",
        "estimated_effort": "medium",
        "benefits": ["Reduced execution coupling", "Better testability"]
      }
    ]
  }
}
```

### AASE Context API

#### GET /api/v1/aase/contexts

List available AASE context artifacts.

**Response:**
```json
{
  "contexts": [
    {
      "id": "CTX-auth-domain-v3",
      "type": "Context",
      "domain": "authentication",
      "version": 3,
      "created_at": "2025-08-03T10:00:00Z",
      "status": "active",
      "dependencies": ["CTX-user-domain-v2"]
    }
  ]
}
```

#### GET /api/v1/aase/contexts/{context_id}

Retrieve specific context artifact.

**Response:**
```json
{
  "context": {
    "id": "CTX-auth-domain-v3",
    "type": "Context",
    "domain": "authentication",
    "version": 3,
    "content": {
      "domain_description": "User authentication and authorization system",
      "key_concepts": ["User", "Session", "Permission", "Role"],
      "business_rules": [
        "Users must authenticate before accessing protected resources",
        "Sessions expire after 24 hours of inactivity"
      ],
      "quality_requirements": {
        "security": "high",
        "performance": "medium",
        "scalability": "high"
      }
    },
    "related_entities": [
      "class_auth_service",
      "func_validate_token"
    ],
    "context_chain": [
      "CTX-user-domain-v2",
      "MDL-user-entities-v1",
      "UCS-login-flow-v2"
    ]
  }
}
```

#### POST /api/v1/aase/generate

Generate new context artifacts.

**Request Body:**
```json
{
  "domain": "payment",
  "artifact_type": "Context",
  "include_connascence": true,
  "base_entities": ["class_payment_service", "func_process_payment"]
}
```

### Real-time Updates API

#### WebSocket /ws/updates

Receive real-time updates about code changes and graph modifications.

**Connection:**
```javascript
const ws = new WebSocket('ws://localhost:8080/ws/updates');

ws.onmessage = function(event) {
    const update = JSON.parse(event.data);
    console.log('Graph update:', update);
};
```

**Message Format:**
```json
{
  "type": "file_changed",
  "timestamp": "2025-08-03T10:30:00Z",
  "data": {
    "file_path": "src/auth/service.py",
    "change_type": "modified",
    "affected_entities": ["func_authenticate_user"],
    "quality_impact": {
      "cohesion_delta": 0.02,
      "new_connascence": []
    }
  }
}
```

## Error Handling

All endpoints return errors in a consistent format:

```json
{
  "error": {
    "code": "INVALID_QUERY",
    "message": "Query syntax is invalid",
    "details": {
      "line": 1,
      "column": 15,
      "suggestion": "Use 'calls' instead of 'call'"
    },
    "request_id": "req_abc123"
  }
}
```

### Error Codes

- `INVALID_QUERY`: Malformed query syntax
- `NODE_NOT_FOUND`: Requested node doesn't exist
- `VERSION_NOT_FOUND`: Requested version doesn't exist
- `PARSE_ERROR`: Failed to parse source code
- `STORAGE_ERROR`: Database or storage failure
- `RATE_LIMIT_EXCEEDED`: Too many requests
- `UNAUTHORIZED`: Invalid or missing API key

## Rate Limiting

- **Query API**: 100 requests/minute per API key
- **Version API**: 50 requests/minute per API key
- **Quality API**: 20 requests/minute per API key
- **WebSocket**: 1 connection per API key

Headers returned:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1691067600
```

## Pagination

Large result sets use cursor-based pagination:

```json
{
  "data": [...],
  "pagination": {
    "has_next": true,
    "cursor": "eyJpZCI6ImFiYzEyMyJ9",
    "total_count": 1500
  }
}
```

Next page request:
```
GET /api/v1/versions?cursor=eyJpZCI6ImFiYzEyMyJ9&limit=50
```

## OpenAPI Specification

Full OpenAPI 3.0 specification available at:
- **JSON**: `/api/v1/openapi.json`
- **YAML**: `/api/v1/openapi.yaml`
- **Swagger UI**: `/api/v1/docs`

## SDK Examples

### Python
```python
import requests

class CodeGraphClient:
    def __init__(self, base_url, api_key):
        self.base_url = base_url
        self.headers = {"Authorization": f"Bearer {api_key}"}
    
    def query(self, question, **kwargs):
        response = requests.post(
            f"{self.base_url}/api/v1/query",
            headers=self.headers,
            json={"question": question, **kwargs}
        )
        return response.json()

# Usage
client = CodeGraphClient("http://localhost:8080", "your-api-key")
result = client.query("What functions have high coupling?")
```

### JavaScript
```javascript
class CodeGraphClient {
    constructor(baseUrl, apiKey) {
        this.baseUrl = baseUrl;
        this.headers = { 'Authorization': `Bearer ${apiKey}` };
    }
    
    async query(question, options = {}) {
        const response = await fetch(`${this.baseUrl}/api/v1/query`, {
            method: 'POST',
            headers: { ...this.headers, 'Content-Type': 'application/json' },
            body: JSON.stringify({ question, ...options })
        });
        return response.json();
    }
}

// Usage
const client = new CodeGraphClient('http://localhost:8080', 'your-api-key');
const result = await client.query('Show me all payment-related functions');
```

### Rust
```rust
use reqwest::Client;
use serde_json::json;

pub struct CodeGraphClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl CodeGraphClient {
    pub async fn query(&self, question: &str) -> Result<serde_json::Value> {
        let response = self.client
            .post(&format!("{}/api/v1/query", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({ "question": question }))
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
}
```

## Performance Considerations

### Caching
- **Query results**: Cached for 5 minutes
- **Version data**: Cached for 1 hour
- **Static assets**: Cached for 24 hours

### Query Optimization
- Use specific filters to reduce result sets
- Limit `max_hops` for relationship queries
- Exclude unnecessary data with `include_*` flags

### Best Practices
- Batch multiple queries when possible
- Use WebSocket for real-time updates instead of polling
- Implement client-side caching for frequently accessed data
- Use pagination for large result sets