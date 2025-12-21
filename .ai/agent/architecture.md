**Project Architecture for AI Agents**

This document is written for automated agents (AI assistants, code generators, bots) that will read, modify, or generate code for this repository. It explains high-level architecture decisions, constraints, and mandatory rules to ensure safe, correct, and maintainable contributions.

**Scope & Goal:**
- Provide a concise reference so agents produce changes that respect crate boundaries, role separation, and the generic trait-based extension model used by the project.

**Core Principles (must-follow)**
- **Crate Segregation is mandatory:** each crate has a single responsibility. Do NOT move code between crates or add responsibilities to a crate outside its intended role. Examples:
  - The `exporter` crate is *only* responsible for exporting metrics/spans. Do not implement exporter logic outside `exporter`.
  - The `probe` crate implements probe types and probe internals. It MUST not contain code that performs exporting or cross-cutting persistence tasks.

- **Role segregation in code:** functions, modules, and types should only perform the role assigned to their crate. If a change would cross a role boundary, create a clear API surface (trait or function) and keep the implementation confined to the appropriate crate.

**Trait-based extension model (required)**
- Each crate exposes a set of canonical, root-level traits that define the extension points for new features (for example: probe types, exporter adapters, resolver plugins). These traits are the single source of truth for generic behavior.
- When adding a new feature, an agent MUST implement these root traits rather than inventing new cross-cutting abstractions. Implementing the crate-root trait ensures the new feature will be discovered and wired automatically (for example, the `engine` crate scans implementations to create pipelines).
- Implementations should live inside the crate native to the feature. The engine or other orchestrator crates should only depend on the trait interfaces (not on concrete implementations directly).

**Priority of rules when generating code (order of enforcement)**
1. Maintain crate responsibilities (never move export logic into non-exporter crates). This is the top rule.
2. Use the root-level traits to add features. If a trait exists that fits the goal, implement it. Prefer this over adding ad-hoc helpers.
3. Do not modify generic traits or generic methods unless explicitly requested by a human reviewer. These traits are the contract used throughout the system; changing them breaks automatic wiring and downstream code.
4. Keep logs, configuration, and wiring consistent with existing patterns and formats used across crates.

**When you think a trait must change**
- Present a short rationale and a minimal compatibility-preserving migration plan as part of your patch (examples: add a new optional method with default implementation, add an associated type instead of changing an existing one).
- Explicit human approval is required before changing root-level traits or generic method signatures.

**Testing & Verification requirements for agents**
- Always run the crate's unit tests or `cargo check -p <crate>` locally (or in the provided CI) after edits.
- If you add implementations of root traits, add minimal unit tests that validate the contract and the engine's automatic discovery where possible.

**Logging & Observability**
- Follow repository conventions for logging/messages (key=value style, `event=` field, `source=` field, etc.).
- Do not add ad-hoc logging formats; align with existing logs to keep parsers and collectors consistent.
- Each function of scraping (crate `probe`) must be instrumented using the tracing crate.

**Code style and minimal invasive changes**
- Prefer small, focused changes that respect the repository's architecture. Avoid sweeping refactors that touch multiple crates unless coordinated with maintainers.

**Summary: rules checklist for AI agents**
- Respect crate responsibilities: yes/no
- Use crate-root trait: yes/no
- Do not change generic trait/method signatures: yes/no (unless explicit user request)
- Add tests and run `cargo check` / `cargo test`: yes/no
- Check with clippy `cargo clippy --all-targets --all-features -- -D warnings`: yes/no
- Keep logging format consistent: yes/no

Follow these rules strictly to produce safe, maintainable code that integrates cleanly with the `engine` and other orchestrator crates. When in doubt, prefer adding an adapter or new implementation inside the correct crate and ask for human confirmation before changing root traits or cross-crate responsibilities.

----

File location: [/.ai/agent/architecture.md](.ai/agent/architecture.md)

**Additional guidance (examples, crate map, CI checks)**

To make the rules immediately actionable, the sections below provide a quick crate responsibility map, concrete trait examples found in this repository, recommended verification commands and a required human-approval template for changes that touch root traits.

**Crate Responsibility Map (quick reference)**
- `probe`: probe implementations, probe metrics, and local probe utilities (icmp, http, etc.). Do NOT export or persist outside probe’s API.
- `exporter`: exporter adapters and export logic (OTLP, Prometheus remote-write, timescale, etc.). All exporter code belongs here.
- `engine`: orchestration, pipeline creation and runtime; it wires probes, discovery, and exporters via root-level traits.
- `configuration`: parsing and configuration models; provides `Parse`/`ParserType` traits and config structs.
- `discovery`: discovery/resolver implementations and interfaces.
- `documentation`, `dashboard`, `api`, etc.: non-core runtime artifacts; do not add runtime logic in these crates.

**Concrete root trait examples (from this repo)**
Implement new features by implementing these traits in the feature's crate. Do not modify these traits unless explicitly allowed.

- `probe::Probe` — file: `probe/src/lib.rs`
  - Purpose: unify probe lifecycle (init, set_targets, scrape, get_metrics).
  - Example usage (implement inside `probe` or a new probe crate):

```rust
// in probe/src/lib.rs (example)
pub trait Probe {
    type Target;
    fn init(name: String) -> Self;
    fn set_targets(&mut self, targets: Vec<Self::Target>);
    fn get_metrics(&self) -> impl Future<Output = Vec<MetricData>> + Send;
    async fn scrape(&self);
}

// New probe implementation lives in the `probe` crate and implements Probe.
```

- `exporter::Exporter` — file: `exporter/src/lib.rs`
  - Purpose: exporter adapter interface used by engine to send metrics/spans out.

- `engine::RunnablePipeline` — file: `engine/src/pipeline.rs`
  - Purpose: runtime pipeline contract used by the engine to run pipelines.

- `engine::PipelineConfig<T>` — file: `engine/src/factory.rs`
  - Purpose: configuration-driven factory trait used to instantiate pipelines automatically.

- `discovery::Discovery` — file: `discovery/src/lib.rs`
  - Purpose: discovery plugin interface to resolve targets dynamically.

- `configuration::Parse<T>` / `ParserType` — file: `configuration/src/lib.rs`
  - Purpose: standardized configuration parsing hooks.

When adding a new probe, exporter or discovery adapter, implement the corresponding trait inside the matching crate. The `engine` crate depends on these traits and will discover and wire implementations automatically.

**Verification and CI commands**
- Run checks for a single crate after edits:

```bash
cargo check -p probe
cargo test -p probe
```

- Run workspace-wide verification:

```bash
cargo check --workspace
cargo test --workspace
```

- If you change code that affects multiple crates run `cargo check --workspace` and the specific crate tests.

**Human approval and trait-change process (required)**
If you believe a root trait must change, follow this process and include the template below in your PR description. A human reviewer must approve before merging.

Process:
1. Propose minimal, compatible changes (prefer adding new optional methods with defaults or adding associated types).
2. Add tests demonstrating the new behavior and a migration plan that keeps older implementations working.
3. Run full workspace checks and include outputs in the PR.
4. Request explicit human approval from a maintainer before merging.

PR description template (include this verbatim when changing root traits):

```
Title: RFC: Change <TraitName> for <reason>

Summary:
- Short rationale for the change.

Backward compatibility and migration plan:
- How existing implementations will continue to work (or steps to update them).

Minimal code diff:
- Show the new trait method signatures and default implementations.

Tests added:
- List of new tests and crates they run in.

CI results:
- Output of `cargo check --workspace` and `cargo test --workspace`.

Request: human approval required before merging.
```

**Final note**
These additions make the original guidance immediately actionable for an AI agent: the crate map reduces ambiguity, trait examples show exactly where to implement features, and CI + approval steps prevent accidental breaking changes to generic contracts.

