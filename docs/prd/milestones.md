# Milestones y Cronograma (MVP orientado a valor ~12 semanas)

Nota: Fechas tentativas; ajustar según disponibilidad del equipo.

- H1 — Grafo mínimo y consultas (fin semana 3)
  - Alcance:
    - Parsers + AST simplificado para Python y JavaScript/TypeScript
    - Relaciones básicas: Calls, Imports, Extends, Implements
    - Graph builder y carga en FalkorDB; CLI analyze + API /query básica
  - Criterios de aceptación:
    - Consultas de llamadas/imports/herencias funcionando
    - P50 < 150ms en dataset de prueba

- H2 — JVM + Versioning + Watcher (fin semana 5)
  - Alcance:
    - Añadir Java y Kotlin
    - File watcher con debounce/batch/ignore y actualización incremental
    - Versionado básico con CAS + Merkle; listar/compare versiones
  - Criterios de aceptación:
    - Re-indexación < 0.5s por cambio (archivo)
    - Compare entrega delta de entidades entre versiones

- H3 — AASE YAML + TracesTo + LLM Context (fin semana 7)
  - Alcance:
    - Ingesta manual de AASE (YAML) para UseCase/Feature/Specification
    - Trazabilidad TracesTo (Code -> Feature/UseCase)
    - Endpoint optimizado para IA: POST /api/v1/query/llm-context
  - Criterios de aceptación:
    - Contexto LLM compacto y priorizado por relevancia; P50 ≤ 300ms
    - Validado en 2-3 escenarios LLM (flujo pago, duplicidad, impacto de eliminación)

- H4 — Calidad inicial + Connascence (Name/Type) (fin semana 9)
  - Alcance:
    - Métricas iniciales de cohesión/coupling/fan-in/out
    - Detectores de connascence: Name y Type
    - Endpoint de overview de calidad
  - Criterios de aceptación:
    - Reporte con hotspots y conteo por tipo (Name/Type)

- H5 — Relaciones avanzadas y AI Context MVP (fin semana 10-11)
  - Alcance:
    - Relaciones: Uses, MemberOf, Realizes, OwnedBy, BoundedContext
    - Endpoints: /context/{id}, /context/search, /context/rank, /context/expand
    - Redacción de PII/secrets y provenance
  - Criterios de aceptación:
    - ≥90% relevancia en golden set interno para context search
    - Expansión de vecindario (1–2 hops) y ranking con pesos

- H6 — Endurecimiento (fin semana 12)
  - Alcance:
    - Tests integrales/property-based, observabilidad y retención
    - Documentación E2E y ejemplo
  - Criterios de aceptación:
    - Build release estable, checklist de contribución cumplido
