# AGENTS.md

## Documentation

Additional documentation is maintained in the `/docs` directory:

| Document | Description |
|----------|-------------|
| [Architecture Overview](docs/architecture.md) | High-level system topology, Rust-Lua relationship, data flow, and design patterns |
| [Rust Core Reference](docs/rust-core.md) | Detailed module breakdown of the Rust engine, component systems, physics pipeline, and rendering pipeline |
| [Performance Requirements](docs/requirements.md) | Target metrics, functional/non-functional requirements, and acceptance criteria for high-entity-count support (Vampire Survivors scale) |
| [Tiered Collision Design](docs/design-tiered-collision.md) | Design doc for range-based tiered collision system (implemented) |
| [Regressions](docs/regressions.md) | Fixed bugs that must not return (root cause, fix, and regression tests) |
