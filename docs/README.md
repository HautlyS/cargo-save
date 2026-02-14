# cargo-save Technical Documentation

This directory contains detailed technical documentation for cargo-save.

## Documentation Structure

### [Architecture Overview](architecture.md)
Complete system design and architecture documentation covering:
- Core components (CacheManager, Hash System, Dependency Graph)
- Data flow and build process
- Parallelization strategy
- Performance characteristics
- Error handling approach
- Extension points for future enhancements

### [Cache System](cache-system.md)
In-depth explanation of the caching mechanism:
- Cache key generation and storage structure
- Build and incremental cache metadata
- Cache validation and invalidation logic
- Hash computation algorithms (git-based and fallback)
- Cache maintenance and cleanup
- Performance optimizations

### [Git Integration](git-integration.md)
How cargo-save uses git for change detection:
- Git commands used (ls-tree, status)
- Change detection algorithm
- Fallback mechanism for non-git projects
- Git status parsing
- Performance characteristics
- Edge cases and limitations

### [CI/CD Integration](ci-integration.md)
Guide for integrating cargo-save with CI/CD systems:
- GitHub Actions setup (basic and advanced)
- GitLab CI configuration
- CircleCI, Jenkins, and Docker examples
- Cache key generation for different platforms
- Best practices and troubleshooting
- Performance comparisons

### [Tool Integration](tool-integration.md)
How to integrate cargo-save with other Rust build tools:
- sccache integration (recommended for cross-project caching)
- cargo-cache integration (disk space management)
- Performance benefits of combining tools
- Setup and configuration guide
- Troubleshooting common issues
- Best practices for maximum performance

## Quick Links

- [Main README](../README.md) - User-facing documentation
- [IMPROVEMENTS.md](../IMPROVEMENTS.md) - Applied improvements summary
- [REVIEW.md](../REVIEW.md) - Code review and future improvements
- [impl.md](../impl.md) - Original implementation notes

## For Developers

If you're contributing to cargo-save, start with:
1. [Architecture Overview](architecture.md) - Understand the system design
2. [Cache System](cache-system.md) - Learn how caching works
3. [REVIEW.md](../REVIEW.md) - See improvement opportunities

## For Users

If you're using cargo-save, check:
1. [Main README](../README.md) - Installation and basic usage
2. [CI/CD Integration](ci-integration.md) - Set up in your CI pipeline
3. [Git Integration](git-integration.md) - Understand change detection

## Technical Specifications

**Language:** Rust 1.70+  
**Hash Algorithm:** Blake3  
**Cache Version:** v3  
**Parallelization:** Rayon  
**Metadata Format:** JSON  

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         cargo-save                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │     CLI      │───▶│ CacheManager │───▶│  Git/Hash    │ │
│  │   (clap)     │    │              │    │  Computation │ │
│  └──────────────┘    └──────────────┘    └──────────────┘ │
│                             │                              │
│                             ▼                              │
│                    ┌──────────────────┐                    │
│                    │ Workspace State  │                    │
│                    │  - Packages      │                    │
│                    │  - Dependencies  │                    │
│                    │  - Hashes        │                    │
│                    └──────────────────┘                    │
│                             │                              │
│                             ▼                              │
│                    ┌──────────────────┐                    │
│                    │ Cache Validation │                    │
│                    │  - Check hashes  │                    │
│                    │  - Verify files  │                    │
│                    └──────────────────┘                    │
│                             │                              │
│                    ┌────────┴────────┐                     │
│                    ▼                 ▼                     │
│            ┌──────────────┐  ┌──────────────┐             │
│            │ Cache Hit    │  │ Cache Miss   │             │
│            │ (skip build) │  │ (run cargo)  │             │
│            └──────────────┘  └──────────────┘             │
│                                      │                     │
│                                      ▼                     │
│                             ┌──────────────────┐           │
│                             │  Save Cache      │           │
│                             │  - Logs          │           │
│                             │  - Metadata      │           │
│                             │  - Artifacts     │           │
│                             └──────────────────┘           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Cache Flow

```
User Command
    │
    ▼
Parse Args ──────────────────────────────────────┐
    │                                            │
    ▼                                            │
Load Workspace Metadata                          │
    │                                            │
    ▼                                            │
Compute Hashes (Parallel)                        │
    │                                            │
    ├─▶ Source Hash (git)                        │
    ├─▶ Cargo.lock Hash                          │
    ├─▶ Environment Hash                         │
    ├─▶ Features Hash                            │
    └─▶ Toolchain Hash                           │
    │                                            │
    ▼                                            │
Build Dependency Graph                           │
    │                                            │
    ▼                                            │
Check Cache for Each Package                     │
    │                                            │
    ├─▶ All Cached? ──Yes──▶ Skip Build ────────┤
    │                                            │
    └─▶ Some Changed? ──Yes──▶ Run Cargo        │
                                │                │
                                ▼                │
                        Capture Output           │
                                │                │
                                ▼                │
                        Save Cache               │
                                │                │
                                └────────────────┤
                                                 ▼
                                            Return Result
```

## Performance Metrics

Typical performance for a 50-package workspace:

| Operation | Time | Notes |
|-----------|------|-------|
| Hash computation | 100-500ms | Parallel, git-based |
| Cache lookup | <1ms | Per package |
| Dependency graph | ~5ms | One-time per build |
| Full workspace scan | 100-500ms | All packages |
| Log query | <1ms | Instant |
| Cache hit (no changes) | <100ms | Skip build entirely |

## Contributing

See [REVIEW.md](../REVIEW.md) for:
- Known issues and limitations
- Improvement opportunities
- Code quality recommendations
- Future enhancement ideas

## License

MIT OR Apache-2.0
