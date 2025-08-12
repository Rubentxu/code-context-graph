# Visualization: Mermaid Class Diagrams

This document explains how to generate Mermaid-based class diagrams from your project using the `ccg` CLI, and how this integrates into the broader goal of providing contextual information (text or images) for LLM workflows.

## Overview
- Project-wide scanning: `ccg viz class` now accepts a directory path and builds a complete class graph for the scanned project.
- Output formats: Markdown/plain Mermaid or full HTML embedding Mermaid.js.
- Filters: include-only by class names; more filters coming (by package/module, depth).
- TDD: All features were developed with tests-first and covered by CLI integration tests.

## Requirements
- Rust toolchain installed
- For HTML output, no extra dependency is required; the generated page loads Mermaid.js from a CDN.
- For future image export (PNG/SVG), we will add a separate dependency (not required yet).

## Usage

### Project-wide class diagram (directory)
Generate a Mermaid class diagram for an entire project directory:

```bash
ccg viz class \
  --path examples/java \
  --out /tmp/java_project.md \
  --format md
```

- `--path` can be a directory or a single file.
- When a directory is given, all supported files are scanned recursively.
- Language is auto-detected per file; unsupported files are ignored gracefully.

### Single file diagram
```bash
ccg viz class \
  --path examples/python/example.py \
  --out /tmp/diagram.md
```

### HTML output (ready to render)
```bash
ccg viz class \
  --path examples/python \
  --out /tmp/python_project.html \
  --format html
```
The generated HTML embeds Mermaid.js with a dark theme and is ready to open in a browser.

### Filters: by class name
```bash
ccg viz class \
  --path . \
  --out /tmp/filtered.md \
  --filter-class User,UserService
```
Only includes the specified classes and relations between them.

## Integration with LLM Context
The visualization feature is part of the broader goal of providing rich project context for LLMs.

Planned enhancements:
- Source from DB: `--source db` to build diagrams from the persisted semantic graph in FalkorDB (after running `analyze`).
- Module/package filters: `--filter-module com.example`.
- Depth limiting: `--max-depth N` for relations.
- Context packs for LLMs: `--emit-context md` to generate a single Markdown artifact with summary sections and multiple Mermaid blocks.
- Image export: `--image png|svg` for LLMs that prefer rendered images.

## Test-driven Development
These capabilities were added with tests-first:
- `crates/cli/tests/viz_cmd_html_and_filters.rs`
- `crates/cli/tests/viz_cmd_project.rs`

They validate:
- HTML output contains a proper `<div class="mermaid">` and loads Mermaid.js.
- Class filtering works for known examples (Java/Python).
- Directory scanning produces a complete class diagram across files.

## Troubleshooting
- If some files are not included, ensure they are in a supported language and readable.
- For very large projects, consider starting with `--filter-class` to scope the diagram.
- DB-based export will be added to support incremental/large-scale analysis via FalkorDB.

## Roadmap
- `--source db` (FalkorDB integration)
- `--filter-module`, `--max-depth`
- `--emit-context md` for LLM packs
- `--image png|svg`
