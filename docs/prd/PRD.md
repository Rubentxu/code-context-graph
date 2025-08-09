# PRD — Code Context Graph

Versión: 1.0
Fecha: 2025-08-09
Estado: Draft

Este PRD consolida los requisitos y el plan de ejecución del proyecto Code Context Graph a partir de la documentación existente: arquitectura, API, configuración, parser, storage, desarrollo y PRD previo.

---

## 1. Resumen Ejecutivo

Code Context Graph es un motor en Rust que analiza código multilenguaje, construye un grafo semántico consultable, calcula métricas de calidad y artefactos AASE, y expone APIs para consumo por humanos y LLMs. Usa CAS + Merkle Trees para versionado eficiente y FalkorDB para almacenamiento de grafo.

- Multilenguaje: Python, JavaScript, Java, Kotlin (extensible)
- Grafo semántico: entidades de código y relaciones (calls, imports, inherits, etc.)
- Actualización incremental + file watch
- Métricas de calidad y connascence
- AASE: artefactos de contexto versionados
- API REST + WebSocket

---

## 2. Objetivos

- Soporte estable para 4 lenguajes (↑ extensible)
- Indexación inicial < 10s en ~500 archivos; re-indexación < 0.5s por cambio
- Cobertura de relaciones ≥ 80% de un repo real
- Respuesta de consultas P50 < 50ms, P99 < 200ms
- Connascence: precisión > 90% en tipos estáticos; sugerencias de refactor
- Versionado con historial ≥ 1000 snapshots sin degradación

KPIs
- % cobertura de relaciones, precisión de consultas de LLM, latencias, deduplicación > 85%, estabilidad (errores/1k consultas)

---

## 3. Alcance

Incluido
- Parsing multilenguaje con Tree-sitter e incremental
- Construcción/consulta de grafo en FalkorDB
- CAS + Merkle para contenido y versiones
- Detección de connascence (estática y dinámica opcional)
- Artefactos AASE (context, model, use_case, prompt, specification)
- API REST, WebSocket para actualizaciones, CLI

Excluido (fase 1)
- Integración con vector stores
- Generación de resúmenes de componentes por LLM integrada
- Autenticación avanzada (OAuth) — usar API key simple inicialmente

---

## 4. Stakeholders y Usuarios

- Devs backend (Rust) — propietarios técnicos
- Devs producto/QA — consumidores de métricas y consultas
- LLM agents — consumidores de contexto y grafo
- Ops — despliegue/monitorización

Personas
- Dev que necesita navegar impacto de cambios
- Arquitecto que analiza acoplamientos, hotspots
- Agente LLM que responde preguntas del repositorio

---

## 5. Requisitos Funcionales

5.1 Ingesta y Parsing
- Detectar lenguaje por extensión/heurísticas
- Limitar tamaño/timeout por archivo (configurable)
- Parsing incremental y por lotes

5.2 Construcción de Grafo
- Entidades: File, Module, Class, Interface, Function/Method, Variable, Type, Enum
- Entidades extendidas: Feature, UseCase, QualityMetric, ContextArtifact
- Relaciones: Contains, Imports, Extends, Implements, Calls, References, Returns, Parameter, Instantiates, Uses
- Relaciones extendidas (negocio y calidad):
  - TracesTo (Code -> Requirement/UseCase/Feature)
  - MemberOf (Artifact -> Feature | Feature -> UseCase)
  - ImplementsFeature (Code/Module -> Feature)
  - Realizes (Feature -> UseCase) [alias de SupportsUseCase con semántica fuerte]
  - DependsOn (Feature -> Feature)
  - SupersetOf / SubsetOf / Overlaps / DisjointWith (entre conjuntos Feature/UseCase/Modules)
  - PartOf / ComposedOf (agregación/descomposición)
  - CohesionWith (intra-conjunto, con peso)
  - CouplesTo (inter-conjunto, con peso y direccionalidad)
  - Satisfies (Feature|Code -> Requirement)
  - Verifies (TestCase -> Feature|UseCase)
  - OwnedBy (Feature|UseCase -> Team)
- Serialización para FalkorDB (MERGE; índices recomendados)

5.3 Connascence y Calidad
- Tipos: Name, Type, Meaning, Position, Algorithm, Execution, Timing (exp), Values, Identity
- Cálculo de fuerza/impacto; umbrales configurables
- Sugerencias de refactor opcionales
- Métricas: cohesión, coupling (afferent/efferent), complejidad, maintainability index

5.4 Versionado y Almacenamiento
- CAS (Blake3, compresión configurable)
- Merkle Trees con fanout configurable; O(log n) diff
- GC y retención configurables

5.5 File Watcher y Actualizaciones
- Debounce, batch threshold, ignore patterns
- Actualización incremental del grafo solo para nodos afectados
- Notificaciones por WebSocket

5.6 API y CLI
- REST: query, graph, versions, quality, aase
- Versiones: listar, comparar, delta de calidad
- Quality: overview, connascence por módulo
- AASE: listar, obtener, generar
- CLI: analyze, watch, query, version

5.7 Seguridad y Validaciones
- API key simple; CORS configurable
- Límite de profundidad de query; sanitización de inputs
- Validación de configuración (schema + runtime)


5.8 Features y Casos de Uso (Modelado, Relaciones de Conjunto y Calidad)

- Nodos:
  - Feature { id, name, description, tags[], owner?, status?, priority? }
  - UseCase { id, name, actor, goal, scenario, pre/post-conditions }
  - Requirement { id, type=functional|nonfunctional, text }
  - TestCase { id, type=unit|integration|e2e }
  - Team { id, name }
  - BoundedContext { id, name, description }
- Trazabilidad:
  - CodeNode -[TracesTo]-> Feature/UseCase
  - Feature -[Realizes]-> UseCase
  - CodeNode/Module -[ImplementsFeature]-> Feature
- Pertenencia jerárquica (superset/subset materializable):
  - Artifact -[MemberOf {evidence, confidence, source}]-> Feature
  - Feature -[MemberOf {evidence, confidence, source}]-> UseCase
- Relaciones de Conjunto (set theory) con propiedades {confidence, evidence, source}:
  - SupersetOf(A,B): A ⊇ B (A cubre todas las entidades de B)
  - SubsetOf(A,B): A ⊂ B
  - Overlaps(A,B): A ∩ B ≠ ∅, A \ B ≠ ∅, B \ A ≠ ∅
  - DisjointWith(A,B): A ∩ B = ∅
  - PartOf(A,B) / ComposedOf(B,A): descomposición funcional
  - DependsOn(A,B): A requiere B (para features y módulos)
- Reglas de evaluación (heurísticas iniciales):
  - Nombre/ruta/paquete (convenciones) → seeds para pertenencia
  - Anotaciones/docs/tests (AASE UseCase/Specification) → evidencia fuerte
  - Consultas de grafo (quórum de relaciones Calls/Imports) → inferencia de pertenencia
  - Umbrales configurables para confidence (porcentaje de code nodes trazados)
- Cohesión y Acoplamiento como relaciones:
  - CohesionWith(X,X): relación auto o entre submódulos de X con weight=[0,1] calculado por métricas (p.ej., densidad de relaciones internas / tamaño)
  - CouplesTo(X,Y): arista dirigida con weight=[0,1], breakdown (afferent/efferent), y metadata {edges_count, instability}
  - Soportes: scope=Module|Feature|UseCase; window=version|moving(N)
- Consistencia y validación:
  - Evitar ciclos inválidos en PartOf/ComposedOf
  - Mantener propiedades de orden parcial en Superset/SubSet
  - Derivar DisjointWith cuando overlap=0 bajo confianza alta

Connascence n-aria y agregación por scope
- Nodo ConnascenceGroup { type, strength, rationale, created_at }
- AffectedBy: Artifact -[AffectedBy]-> ConnascenceGroup (n-aria a través de hub)
- Proyección/Agregación:
  - AggregateConnascence(scope=Feature|UseCase|Module) calculada a partir de sus miembros con ventana (version|moving N)
  - Persistencia opcional de agregados por versión para consultas rápidas

Contextos de dominio
- Feature -[MemberOf]-> BoundedContext
- UseCase -[Touches]-> BoundedContext (cuando cruza límites)

Notas de implementación:
- Entrada primaria de Features/UseCases desde artefactos AASE (use_case/specification) y ficheros de catálogo opcionales (features.yaml)
- Enriquecimiento automático por reglas y aprendizaje incremental de pertenencia (futuro)

---

## 6. Requisitos No Funcionales

- Rendimiento: ver objetivos de latencia; uso de Tokio/Rayon; LRU caches
- Escalabilidad: particionado de storage; batch sizes configurables
- Concurrencia: lock-free donde sea posible; backpressure
- Observabilidad: logging estructurado; métricas opcionales Prometheus; health checks
- Portabilidad: Linux/macOS/Windows

---

## 7. Arquitectura (resumen)

- Hexagonal: domain, application (parser/graph/connascence/AASE), infra (storage/watcher/api), interface (CLI)
- Dataflow: Source → Parser → AST → Graph Builder → FalkorDB → Query Engine → LLM Context; File Watcher → CAS/Merkle → Graph
- Extension points: nuevos lenguajes, detectores de calidad, backends de storage y graph

Referencias: `docs/architecture.md`, `docs/storage.md`, `docs/parser.md`.

---

## 8. API (resumen)

- POST /api/v1/query — preguntas NL o estructuradas sobre el grafo
- GET /api/v1/graph/nodes/{id} — detalle de nodo + relaciones
- GET /api/v1/versions — lista de versiones; POST /compare — delta
- GET /api/v1/quality/overview — métricas generales
- POST /api/v1/quality/connascence — análisis focalizado
- GET /api/v1/aase/contexts — listar; GET /{id} — obtener; POST /generate — crear
- GET /api/v1/domain/features — listar/filtrar features; GET /{feature_id}
- GET /api/v1/domain/use-cases — listar/filtrar casos de uso; GET /{use_case_id}
- POST /api/v1/domain/relations/sets — calcular/consultar Superset/Subset/Overlaps/DisjointWith
- GET /api/v1/quality/relations — consultar CohesionWith/CouplesTo por scope (Module|Feature|UseCase)
 - GET /api/v1/domain/features/{id}/artifacts — miembros con evidence/confidence
 - GET /api/v1/domain/use-cases/{id}/features — features que realizan el caso de uso y sus métricas
 - GET /api/v1/quality/connascence/aggregate?scope=Feature|UseCase&id=...&window=...
 - POST /api/v1/domain/sets/derive — derivar/memorializar relaciones de conjunto con thresholds
- WS /ws/updates — notificaciones en tiempo real

Referencias: `docs/api.md`.

---

## 9. Configuración (resumen)

- Fuentes: CLI > ENV > archivo > defaults
- Áreas: engine, parser, falkordb, cas, file_watcher, versioning, connascence, aase, quality_metrics, api, logging, monitoring, security
- Env prefix: CCG_

Referencias: `docs/configuration.md`.

---

## 10. Métricas de Éxito y Aceptación

Criterios de Aceptación (MVP)
- Indexa 4 lenguajes y construye grafo consultable en FalkorDB
- API responde queries de llamadas, imports, jerarquías, y hotspots de calidad
- Watcher actualiza incrementalmente con latencias objetivo
- Versionado operativo: listar y comparar versiones con delta de calidad
- Connascence básico operativo con umbrales y reporte
- Se modelan Features y UseCases como nodos; existe trazabilidad Code->Feature/UseCase
- Se pueden consultar relaciones de conjunto (superset/subset/overlaps/disjoint)
- Se exponen CohesionWith y CouplesTo con pesos y alcance configurable
 - MemberOf activo para pertenencia Artifact->Feature y Feature->UseCase con evidence/confidence
 - Endpoints para listar artifacts por feature y features por use case
 - Agregación de connascence por Feature/UseCase materializada por versión

Métricas
- Cobertura de relaciones ≥ 80%
- P50/P99 de consultas bajo objetivos
- Deduplicación de storage ≥ 85%
- Incidencias críticas por semana: tendencia decreciente

---

## 11. Roadmap y Entregables

Fase 0 — Fundaciones (semana 1-2)
- [ ] Esqueleto de crates, config validada, logging/tracing
- [ ] CAS + Merkle mínimo viable; CLI analyze básica

Fase 1 — Grafo y Parsing (semana 3-5)
- [ ] Parsers y AST simplificado para 4 lenguajes
- [ ] Graph builder + inserción FalkorDB; queries básicas
- [ ] Índices en grafo para performance inicial

Fase 2 — Watcher e Incremental (semana 6-7)
- [ ] File watcher con debounce/batch/ignore
- [ ] Actualización incremental del grafo
- [ ] Versioning integrado en pipeline

Fase 3 — Calidad y Connascence (semana 8-9)
- [ ] Métricas de calidad + overview API
- [ ] Detectores de connascence; umbrales; reporte
- [ ] Sugerencias de refactor iniciales
- [ ] Modelado de CohesionWith/CouplesTo como aristas con pesos y breakdown
 - [ ] ConnascenceGroup n-aria y agregación por Feature/UseCase

Fase 4 — AASE y APIs (semana 10-11)
- [ ] Artefactos AASE (listar/obtener/generar)
- [ ] API REST y WS estables; rate limiting y CORS
- [ ] CLI: query/version/watch mature
- [ ] Ingesta de Features/UseCases desde AASE y catálogo; trazabilidad Code->Feature/UseCase
- [ ] Cálculo y consulta de relaciones de conjunto (Superset/Subset/Overlaps/Disjoint)
 - [ ] Endpoints: artifacts por feature; features por use case; aggregate connascence

Fase 5 — Endurecimiento (semana 12)
- [ ] Pruebas integrales, property-based, rendimiento
- [ ] Observabilidad y GC/retención
- [ ] Documentación y ejemplo end-to-end

Hitos
- H1: Grafo básico consultable
- H2: Incremental + versiones
- H3: Calidad + connascence
- H4: AASE + APIs completas

---

## 12. Riesgos y Mitigaciones

- Rendimiento en repos grandes → particionado, batch, índices, caches
- Inconsistencia en parsers → pruebas doradas por lenguaje, tolerancia a errores
- Cambios de schema en grafo → migraciones versionadas, compatibilidad
- Complejidad de incremental → pruebas sobre flujos de edición reales
- Seguridad de API → límites, sanitización, API key, CORS

---

## 13. Dependencias Externas

- FalkorDB/RedisGraph
- Tree-sitter y parsers por lenguaje
- Tokio, Rayon, sled/zstd, tracing

---

## 14. Anexos

- Modelos de datos y estructuras: ver `docs/parser.md`, `docs/storage.md`
- Ejemplos de requests/responses: ver `docs/api.md`
- Guía de desarrollo y contribución: `docs/development.md`

### 14.1 Ejemplo de respuesta para LLM (referencia)

```json
{
  "query": "¿Qué hace la función processPayment?",
  "context": {
    "function": {
      "name": "processPayment",
      "file": "payment_service.py",
      "calls": ["validateCard", "chargeAmount", "sendReceipt"],
      "called_by": ["checkoutOrder", "subscriptionRenewal"],
      "parameters": ["amount", "card_info", "user_id"],
      "description": "Procesa pagos validando tarjeta y ejecutando cargo"
    },
    "quality_metrics": {
      "complexity": 8,
      "cohesion": 0.75,
      "coupling": { "afferent": 2, "efferent": 5, "instability": 0.71 }
    },
    "connascence": [
      { "type": "Execution", "with": "validateCard", "strength": 0.9 },
      { "type": "Type", "with": "PaymentGateway", "strength": 0.4 }
    ]
  }
}
```

### 14.2 Ejemplo de grafo (Cypher/GraphQL-like)

```graphql
(:Function {id: "foo", language: "Python", file: "services/auth.py"})
(:Class {id: "AuthService", language: "Python", file: "services/auth.py"})
(:foo)-[:CALLS {line: 45}]->(:bar)
(:AuthService)-[:DEFINES]->(:foo)
```

### 14.3 Tipos de nodos y relaciones (resumen)

- NodeType: File, Module, Class, Interface, Function, Method, Variable, Type, Enum, ConnascenceNode, ContextArtifact, QualityMetric, Feature, UseCase
- RelationType: Contains, Imports, Extends, Implements, Calls, References, Returns, Parameter, Instantiates, Uses, HasConnascence, TracesTo, ImplementsFeature, SupportsUseCase, DependsOn, PartOf, ComposedOf, SupersetOf, SubsetOf, Overlaps, DisjointWith, CohesionWith, CouplesTo

---

## 15. Viabilidad técnica y enfoque de extracción en Rust

Resumen: Es viable implementar en Rust la extracción del grafo de clases/artefactos, relaciones y grados/tipos de relación, así como la detección de connascence estática (y dinámica opcional) usando Tree-sitter, análisis semántico ligero y agregación por versiones. Se detallan pipeline, heurísticas por lenguaje, componentes y riesgos.

15.1 Pipeline de extracción
- File → AST (Tree-sitter) → AST simplificado → Indexación (símbolos, scopes) → Resolución (nombres, tipos básica) → Relaciones (calls/imports/extends/implements/references/defines) → Call/Class Graph → Detección de Connascence → Pesos/Métricas → Persistencia (CAS/Merkle + FalkorDB)
- Incremental: diffs de AST y alcance de impacto para recalcular solo nodos/relaciones afectadas.

15.2 Grafo de clases y relaciones
- Clases/Interfaces/Módulos/Funciones/Variables extraídos del AST con ubicación y metadatos.
- Relaciones base:
  - Contains/Defines: por estructura del AST.
  - Imports/Uses/References: por nodos import/identifier binding; requiere tabla de símbolos por archivo y módulo.
  - Extends/Implements: por nodos de herencia/implements.
  - Calls: edges desde call-sites hacia funciones/métodos resueltos; si hay sobrecarga, usar nombre calificado + heurística de tipo.
  - Returns/Parameter/Instantiates: por firmas y expresiones new/construct.
- Grados/tipos de relación y pesos:
  - edge.weight = función(line_count, frecuencia, fan-in/out, proximidad, estabilidad de dependencia).
  - CouplesTo y CohesionWith se derivan agregando weights intra/inter conjunto (Module|Feature|UseCase).

15.3 Detección de Connascence (estática y dinámica opcional)
- Estática (viable hoy con AST + reglas): Name, Type, Meaning, Position, Algorithm.
  - Name: identificadores duplicados con acoplamiento lógico (nombres compartidos en contratos/convenciones).
  - Type: coincidencia de tipos entre productores/consumidores (firmas, genéricos simples, aliases).
  - Meaning: valores mágicos/constantes compartidas; detección por literales repetidos y enums.
  - Position: orden de parámetros consistente entre call-sites y definiciones.
  - Algorithm: duplicación estructural (hash de subárboles AST) y llamadas en patrón.
- Dinámica (opcional futuro): Execution/Timing a partir de trazas de tests o instrumentación ligera.
  - En Rust: usar cargo test con instrumentation; en JVM/Node/Python: hooks de runtime/pruebas.
- Modelo n-ario: ConnascenceGroup como hub con edges AffectedBy a cada artefacto involucrado.
- Fuerza/impacto: calcular score ∈ [0,1] por tipo con umbrales configurables y evidencia (tests/docs/annotations).

15.3.1 Adquisición de connascence de runtime (Execution/Timing)
- Objetivo: Capturar co-ejecuciones y dependencias temporales entre artefactos para inferir connascence de Execution y Timing.
- Estrategia por lenguaje:
  - Python: sys.setprofile / sys.settrace, pytest plugin, cobertura (coverage.py) y mapping a funciones; opcional eventos de asyncio.
  - Node.js/TS: V8 Inspector Protocol (Tracing/Profiler), hooks async, Istanbul cobertura; map a símbolos via sourcemaps (TS).
  - Java/Kotlin (JVM): Java Agent con ByteBuddy/ASM para instrumentar entry/exit; opcional JFR/JVM TI; integración JUnit.
  - Rust: tracing crate (instrument), cargo-llvm-cov para cobertura; macros #[instrument] en funciones objetivo.
- Datos capturados (evento): {ts, thread_id, span_id, file, symbol, qualified_name, event=enter|exit, args_hash?, alloc_id?}
  - Opcional: lectura/escritura de estado (shared var id), tamaño de payload, latencia.
- Ingesta:
  - POST /api/v1/runtime/traces — batch NDJSON/Protobuf con metadatos {commit, version_id, test_id, run_id, env}.
  - Normalización y correlación a nodos del grafo por (qualified_name, file, line_range) y tablas de símbolos por versión.
- Derivación:
  - Co-ejecución: pares de símbolos en la misma traza/test con frecuencia > τ → ConnascenceGroup(type=Execution).
  - Timing: secuencias con dependencia temporal estable (A precede B con p>τ) → ConnascenceGroup(type=Timing).
  - Peso/fuerza: función de frecuencia relativa, cobertura, latencia media y estabilidad entre runs.
- Agregación y materialización:
  - Agregar por Feature/UseCase/Module por versión; persistir snapshots para consultas rápidas.
  - Marcar confidence según cobertura de tests y consistencia entre runs.
- Controles de performance/privacidad:
  - Sampling de spans, límites por paquete/proyecto, exclusiones por path.
  - Redacción de datos sensibles en args; no persistir payloads.
- Reproducibilidad:
  - Registrar {commit, test_id, seed, env} para repetir runs; cachear correlaciones por version_id.

15.4 Estrategia por lenguaje (resolución y límites)
- Python: dinámica; buena cobertura con Tree-sitter + heurísticas de import/module y nombres calificados; tipos opcionales (type hints) si existen.
- JavaScript/TypeScript: para TS, usar tipo básico mediante lectura de declaraciones; para JS, heurísticas de módulo (ESM/CommonJS) y patrones; opcional integración tsserver para mejora futura.
- Java: requerirá resolución de tipos simplificada (paquetes/imports, firmas); viable con Tree-sitter-java + índice de clases; para precisión avanzada, evaluar integración JDT/bytecode en fase posterior.
- Kotlin: similar a Java con peculiaridades de top-level functions y extensiones; resolver por paquete/archivo y firmas.

15.5 Crates y herramientas
- tree-sitter, tree-sitter-<lang>, tree-sitter-traversal
- blake3, zstd, sled (CAS), rayon/tokio para concurrencia
- petgraph para cálculos de grafos locales; cliente Redis/FalkorDB para persistencia
- regex, aho-corasick para detecciones textuales; similar-string para duplicidad ligera
- optional: sourcemap/tsserver bindings, JDT/ASM para JVM en fases posteriores

15.6 Complejidad, rendimiento y almacenamiento
- Complejidad: O(N) por archivo para parsing y extracción; relaciones transversales mediante índices hash.
- Incremental: reuso de subárboles AST y reconstrucción de relaciones locales; actualización de agregados por delta.
- Almacenamiento: CAS con deduplicación; relaciones en grafo con índices por (type,name), (file->symbols), (symbol->refs).

15.7 Riesgos y mitigaciones
- Resolución de tipos incompleta en lenguajes dinámicos → usar heurísticas + evidencia múltiple; marcar confidence en edges.
- Sobrecarga/dispatch en OOP → desambiguación por firmas y contexto; si ambiguo, varios candidatos con pesos.
- Performance en repos grandes → batch, índices, materialización de agregados por versión, caches LRU.
- Calidad de parsers/gramáticas → tests dorados por lenguaje y resiliencia ante errores.
- Integraciones externas (tsserver/JDT) → opción post-MVP, detrás de feature flags.

15.8 Conclusiones
- Viabilidad: Alta para extracción estática de grafo y connascence estática con Tree-sitter y reglas en Rust.
- Valor incremental: dinámica (Execution/Timing) posible vía instrumentación de tests en iteraciones futuras.
- Trade-off: priorizar precisión suficiente con confidence/evidence y agregación por versión para decisiones de arquitectura.

---

## 16. Context Engineering para Agentes de IA

Objetivo: Enriquecer cada entidad y relación del grafo con contexto útil para agentes de IA (RAG/Toolformer/Planner) que consuman este grafo. Proveer metadatos, sumarios, evidencia, embeddings y APIs de recuperación/ranking que combinen señales semánticas y estructurales del grafo.

16.1 Facetas de contexto por Nodo y Arista
- Comunes (Node y Edge):
  - summary: resumen extractivo/abstractive del propósito.
  - intent/purpose: objetivo funcional o responsabilidad.
  - invariants/contracts: pre/postcondiciones, restricciones, tipos.
  - examples/snippets: uso canónico, I/O, casos límite.
  - evidence/provenance: fuente (archivo/commit/test/doc), versión, timestamp.
  - confidence: [0,1], método (rule|heuristic|llm|human), sample_size.
  - tags: domain, bounded_context, risk, priority, status.
  - security/licensing: PII?, secrets?, licencia, policy flags.
  - embeddings: ids para vector store por modalidad (code/doc/test), dim, model.
- Específicos de Nodo:
  - NodeType=Code: signature, complexity, fan-in/out, hotspot score, owners.
  - NodeType=Feature/UseCase: actor, goal, acceptance_criteria, KPIs.
  - NodeType=Requirement/TestCase/Team/BoundedContext: campos propios y enlaces.
- Específicos de Arista:
  - type, weight, rationale, breakdown (para CouplesTo/CohesionWith), direction.
  - justification: por qué existe (pattern, rule, commit, refactor).

16.2 Pipeline de enriquecimiento
- Extracción primaria (sección 15) produce AST, símbolos, relaciones y métricas.
- Enriquecimiento:
  - Resumen extractivo: seleccionar oraciones/código relevantes (TextRank/TF-IDF, límites por tokens);
  - Resumen abstractive (opcional): vía proveedor LLM externo, detrás de feature flag.
  - Keywords y tópicos: n-grams, RAKE/KeyBERT-like (sin dependencia pesada), tags heurísticas.
  - Evidencia y procedencia: enlazar commits, tests que ejercitan el nodo, docs cercanas.
  - Embeddings (opcional): generar y almacenar en vector store; si no hay, fallback a búsqueda léxica + estructura.
- Incremental: recalcular facetas solo para nodos/edges afectados por cambios.

16.3 Almacenamiento e indexación
- Metadatos de contexto como propiedades en grafo + blobs en CAS (resúmenes largos, snippets, rationales).
- Vector store opcional: Redis Search/Redis-VSS, Qdrant o Tantivy+HNSW local. Guardar reference_id en el nodo/arista.
- Índices:
  - por (node_type, tags[]), full-text en summary/intent, by bounded_context/feature/use_case.
  - vector index por modalidad (code/doc/test) si habilitado.

16.4 Recuperación y ranking para IA
- Scoring combinado (final_score ∈ [0,1]):
  - semantic_score (BM25/embedding) × α
  - graph_proximity (distancia en grafo al seed/consulta) × β
  - relation_weight (fan-in/out, CouplesTo/CohesionWith) × γ
  - recency/version_weight × δ
  - evidence_confidence × ε
  - policy_penalties (PII/secret/licensing) → redacción/filtrado
- Estrategias:
  - Neighborhood expansion con límites por hop y tipo.
  - Proyección por Feature/UseCase/BoundedContext.
  - Deduplicación y diversidad (MMR) para paquetes de contexto.

16.5 APIs orientadas a IA
- GET /api/v1/context/{node_id} — contexto enriquecido del nodo/arista.
- POST /api/v1/context/search — consulta híbrida (texto + filtros + seeds de grafo) → top-K con trazas.
- POST /api/v1/context/rank — re-rank de candidatos con pesos α..ε configurables.
- GET /api/v1/context/expand?seed=...&hops=...&types=... — expansión de vecindario.
- GET /api/v1/context/prompts/{task} — plantillas de prompts y guardrails por tarea.
- POST /api/v1/context/embeddings — gestión de embeddings (crear/actualizar/listar) si vector store habilitado.

16.6 Gobernanza y seguridad
- Redacción automática de PII/secrets (regex/heurísticas) con anotación de redacción.
- Flags de licencia/uso y restricciones por endpoint.
- Auditoría: provenance obligatorio y trazabilidad de cómo se generó cada resumen/embedding.

16.7 Criterios de aceptación (AI Context MVP)
- Para nodos Code/Feature/UseCase: summary/intent/evidence/confidence disponibles vía API.
- Context search híbrida devuelve >=90% de resultados relevantes en golden set interno.
- Ranking configurable por pesos α..ε; soporte de expansión por 1-2 hops con límites.
- Redacción básica de secretos/PII activa por defecto.
- Embeddings y vector store opcionales; si deshabilitados, desempeño aceptable con BM25+grafo.

16.8 UseCase Context Bundle
- Objetivo: Dado un UseCase, devolver un paquete autocontenible de contexto útil para un agente de IA.
- Contenido del bundle:
  - use_case: metadata + summary + intent + acceptance_criteria + KPIs + bounded_context + owners.
  - features: lista con summary, status, prioridad, owners, Realizes links.
  - artifacts implicados: clases, funciones, módulos (MemberOf->Feature) con firmas, snippets, complejidad, fan-in/out, owners.
  - relaciones clave: Calls/Imports/Extends/Implements/Uses entre artefactos del scope.
  - connascence: tipos presentes (Name/Type/Meaning/Position/Algorithm), grupos (ConnascenceGroup) y scoring agregado para el scope.
  - calidad: CohesionWith/CouplesTo dentro y fuera del scope, con breakdown y recomendaciones iniciales.
  - prompts: plantillas específicas para tareas comunes (explaining, refactor, test-gen) con variables rellenadas del bundle.
  - evidence/provenance: commits relevantes, tests que ejercitan el scope, docs asociadas.
- Endpoint:
  - GET /api/v1/context/use-cases/{id}/bundle?depth=...&include=features,artifacts,relations,connascence,quality,prompts,evidence
  - Devuelve JSON con trazas (explainability) y referencias a blobs (CAS) para textos largos/snippets.

16.9 Actualizaciones incrementales y API de mutación
- Requisitos: reflejar cambios frecuentes del código (añadir/actualizar/borrar) en entidades y relaciones; mantener bundle coherente por versión.
- Arquitectura:
  - Watcher → Pipeline incremental (sección 15) → Delta de nodos/aristas → Upsert/Soft-delete en FalkorDB.
  - Versionado: cada delta referencia version_id (CAS/Merkle root); queries pueden fijar versión.
  - Idempotencia: operaciones MERGE por claves naturales (path+signature para code; id para dominio).
  - Consistencia eventual: agregados (connascence/cohesión/coupling) se recalculan por delta y se materializan por versión.
- API de mutación (para dominios/AASE y metadata de contexto; el código se actualiza vía watcher):
  - POST /api/v1/domain/{entity} (Feature|UseCase|Requirement|Team|BoundedContext)
  - PATCH /api/v1/domain/{entity}/{id}
  - DELETE /api/v1/domain/{entity}/{id}
  - POST /api/v1/context/metadata/{node_id} (actualizar summaries/intent/tags/evidence; con provenance)
  - POST /api/v1/context/prompts/{task} (crear/actualizar plantillas)
- Notificaciones:
  - WS /ws/updates emite eventos NODE_UPSERT, EDGE_UPSERT, NODE_DELETE, EDGE_DELETE, AGGREGATE_UPDATE con version_id.
- Criterios:
  - Tiempos objetivo: detección→reflejo en grafo < 1s por archivo; bundle actualizado bajo la misma versión.
  - Operaciones de mutación con retry/idempotencia y validaciones (referential integrity, policy checks).

---

## 17. Enriquecimiento asistido por LLM (adaptadores)

Motivación: Ciertas piezas de contexto y calidad (p.ej., matices de Meaning/Algorithm en connascence, intent detallado, invariants implícitas, resúmenes de alto nivel) no siempre se pueden extraer de forma fiable solo con análisis estático en Rust. Proponemos un mecanismo opcional y acotado para delegar a un LLM la generación/complemento de estos datos, con controles de seguridad, coste y trazabilidad.

17.1 Alcance del LLM
- Generar/actualizar: summary, intent/purpose, invariants/contratos propuestos, ejemplos/snippets explicativos.
- Enriquecer connascence: rationale/explicación y clasificación fina donde falte señal; nunca sustituir evidencia objetiva.
- Sugerencias de refactor/responsibility hints para alta coupling/baja cohesión (marcadas como hints).

17.2 Principios y reconciliación
- Programmatic-first: los datos extraídos determinísticamente (alta confianza) prevalecen; el LLM solo rellena huecos o añade anotaciones con menor prioridad.
- Confianza y procedencia: todo output del LLM se persiste con provenance {method: "llm", provider, model, prompt_hash, created_at} y confidence distribuida aparte.
- Reconciliación: reglas por campo (e.g., no sobreescribir summaries humanos aprobados; combinar invariants como conjunto; rationale solo anexa).
- Human-in-the-loop opcional: cola de revisión para aprobar/rechazar anotaciones sensibles.

17.3 Arquitectura de adaptadores
- Abstracción Provider: OpenAI, Anthropic, Azure, Llama.cpp/Local. Config por feature flag.
- Orquestación: jobs asincrónicos en cola (batch) disparados por:
  - cambios detectados (delta) y thresholds (p.ej., archivo > N líneas cambiadas);
  - solicitud explícita (API) o curación manual.
- Contexto: se construye un paquete mínimo (código/snippets, firmas, relaciones cercanas, tests, docs) con límites de tokens; referencias largas se guardan en CAS y se adjuntan como URLs.
- Caching: clave por (prompt_template_id, context_hash, model) con TTL; guardar en CAS para reuso.
- Coste y cuotas: budgets por día/proyecto y métricas; rechazar/posponer si se excede.

17.4 Seguridad y cumplimiento
- Redacción previa y posterior (PII/secrets) en el contexto y en las respuestas.
- Políticas por licencia/privacidad: evitar envío de fragmentos con restricciones; lista de exclusiones (paths, repos).
- Telemetría y auditoría: log estructurado de prompts y decisiones (con hashes, no contenido sensible).

17.5 APIs
- POST /api/v1/enrich/{scope} — scope ∈ Node|Edge|Feature|UseCase|Bundle; body con ids, campos objetivo y estrategia (extractive|abstractive|hints). Devuelve job_id.
- GET /api/v1/enrich/jobs/{job_id} — estado/resultados parciales.
- GET /api/v1/review/queue — ítems pendientes de aprobación (si HI-LO activado).
- POST /api/v1/review/{item_id}/approve|reject — aplicar/descartar anotaciones.

17.6 Criterios de aceptación
- Feature flag para activar LLM por ámbito (global/proyecto) y por tipo de campo.
- Provenance obligatorio y cache efectivo (hit-rate ≥ 60% en re-ejecuciones de CI sobre el mismo commit).
- Cost guard-rails: hard budget diario y backoff exponencial respetados.
- Reconciliación estable: no se sobreescriben campos deterministas ni aprobados manualmente; conflictos se marcan para revisión.
- Seguridad: redacción aplicada y ausencia de filtraciones en pruebas; auditoría disponible.
