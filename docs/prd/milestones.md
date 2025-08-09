# Milestones y Cronograma (MVP ~12 semanas)

Nota: Fechas tentativas; ajustar según disponibilidad del equipo.

- H1 — Grafo básico consultable (fin semana 5)
  - Alcance:
    - Parsers + AST simplificado para 4 lenguajes
    - Graph builder y carga en FalkorDB
    - Consultas básicas vía CLI/API
  - Criterios de aceptación:
    - Consultas de calls/imports/herencias funcionando
    - Índices iniciales aplicados; P50 < 100ms en dataset de prueba

- H2 — Incremental + Versiones (fin semana 7)
  - Alcance:
    - File watcher con debounce/batch/ignore
    - Actualización incremental del grafo
    - CAS + Merkle integrados; listar/compare versiones
  - Criterios de aceptación:
    - Re-indexación < 0.5s por cambio
    - Compare entrega delta de calidad y entidades

- H3 — Calidad + Connascence (fin semana 9)
  - Alcance:
    - Métricas de cohesión/coupling/complexidad
    - Detectores de connascence (estática + ejecución)
    - Endpoint de overview de calidad
  - Criterios de aceptación:
    - Reporte con hotspots y conteo por tipo de connascence

- H3.5 — AI Context MVP (fin semana 10)
  - Alcance:
    - Facetas de contexto: summary, intent, evidence/provenance, confidence, tags
    - Búsqueda híbrida (lexical+grafo; embeddings opcional) y ranking configurable
    - Endpoints: /context/{id}, /context/search, /context/rank, /context/expand
    - Redacción básica de PII/secrets y auditoría de procedencia
  - Criterios de aceptación:
    - Contexto disponible para Code/Feature/UseCase vía API
    - ≥90% relevancia en golden set interno para context search
    - Expansión de vecindario (1–2 hops) y ranking con pesos α..ε
    - Redacción por defecto activa

- H4 — AASE + APIs completas (fin semana 11)
  - Alcance:
    - Artefactos AASE listar/obtener/generar
    - API REST estable + WebSocket + rate limiting/CORS
    - CLI madura (query/version/watch)
  - Criterios de aceptación:
    - Flujos E2E documentados, P50 < 50ms en consultas típicas

- H5 — Endurecimiento (fin semana 12)
  - Alcance:
    - Tests integrales/property-based
    - Observabilidad y GC/retención
    - Documentación y ejemplo E2E
  - Criterios de aceptación:
    - Build release estable, checklist de contribución cumplido
