#### compiler *win compatible
Drop into any directory containing Rust, Go, TypeScript, C/C++, Python, or Zig projects and `compiler` will discover them, resolve inter-project dependencies via topological sort, build independent projects in parallel, lint with your preferred toolchain, and optionally feed errors to an LLM for automated fixes with rollback safety.

## Features

- **Polyglot Detection**: Walks the directory tree and identifies projects by manifest files (Cargo.toml, go.mod, package.json, CMakeLists.txt, Makefile)
- **Dependency-Aware Ordering**: Parses manifests for path/local dependencies and topologically sorts the build graph, with cycle detection fallback
- **Parallel Builds**: Independent projects within the dependency graph are built concurrently via Rayon using proper topological-level grouping
- **Build Caching**: Hashes source files with SHA-256 and skips rebuilds when nothing has changed
- **Unified Linting**: Delegates to per-language linters (Clippy, golangci-lint, Biome, clang-tidy, Ruff, zig build) through a common plugin interface
- **AI-Powered Fixes**: Sends build/lint diagnostics to Ollama, Anthropic, or any OpenAI-compatible endpoint; batches errors per-file for efficient LLM usage; rolls back garbage responses
- **Watch Mode**: Uses `ReadDirectoryChangesW` via the `notify` crate with a polling fallback for automatic rebuild/lint on save
- **Plugin Architecture**: Each language is a `Plugin` trait implementation, making new languages a single-file addition
- **Configurable Toolchains**: Override runtimes, compilers, linkers, package managers, and linters via TOML config or CLI flags
- **JSON Output**: Machine-readable output for CI/CD integration
- **Shell Completions**: Auto-generated completions for PowerShell, Bash, Zsh, Fish, and Elvish
- **Config Validation**: Warns on unknown sections and invalid provider names in `.compiler/config.toml`
- **Dependency Graph Visualization**: ASCII or Graphviz DOT output of the project dependency graph
- **No System OpenSSL**: Ships with `rustls` so it runs as a standalone `compiler.exe` with no native crypto dependencies

## Supported Languages

| Language | Manifest | Compiler/Runtime | Linter | Build System |
|---|---|---|---|---|
| Rust | `Cargo.toml` | cargo | clippy | cargo |
| Go | `go.mod` | go | golangci-lint | go build |
| TypeScript | `package.json` | bun/node/deno | biome/eslint | bun/npm/yarn/pnpm |
| C/C++ | `CMakeLists.txt`, `Makefile` | clang/gcc | clang-tidy | cmake/make |
| Python | `pyproject.toml` | uv/pip/poetry/pdm | ruff | uv/pip/poetry/pdm |
| Zig | `build.zig` | zig | zig build | zig build |

## Install

```powershell
git clone https://github.com/alexph10/compiler.git
cd compiler
cargo build --release
Copy-Item .\target\release\compiler.exe "$env:USERPROFILE\.cargo\bin\"
```

Make sure `%USERPROFILE%\.cargo\bin` is on your `PATH`.

## Usage

```powershell
compiler build              # detect and build all projects
compiler build --test       # build then run tests
compiler build --run        # build then run the artifact
compiler build --lint       # build then lint
compiler build --clean      # clean before building
compiler build --fix        # build, lint, then AI-fix errors
compiler build --release    # optimized build

compiler test               # run tests across all projects
compiler test --filter foo  # filter tests by name pattern

compiler lint               # lint all detected projects
compiler lint --fix         # auto-fix lint issues

compiler clean              # remove build artifacts

compiler fix                # build + lint + AI-fix in one pass
compiler fix --provider anthropic --model claude-sonnet-4-20250514
compiler fix --max-fixes 5  # limit to fixing 5 files

compiler init               # generate .compiler/config.toml from detected projects
compiler status             # show project status, toolchains, and config
compiler graph              # print dependency graph (ASCII)
compiler graph --dot        # print dependency graph (Graphviz DOT)

compiler watch              # rebuild on file changes
compiler watch --test       # rebuild and test on changes
compiler watch --lint       # rebuild and lint on changes

compiler completions powershell  # generate shell completions
compiler completions bash
compiler completions zsh
compiler completions fish
```

Global flags apply to all subcommands:

```powershell
compiler --runtime deno build          # override TS runtime
compiler --package-manager pnpm build  # override TS package manager
compiler --compiler gcc build          # override C/C++ compiler
compiler --linker mold build           # override Rust linker
compiler --runner poetry build         # override Python runner
compiler --linter eslint lint          # override linter
compiler --json build                  # output results as JSON
compiler --filter rust build           # only build Rust projects
compiler --only .\my-app build         # only build specific project
compiler --quiet build                 # suppress non-error output
compiler --verbose build               # verbose output
compiler --no-color build              # disable ANSI colors
```

## Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Build failure |
| 2 | Lint failure |
| 3 | AI fix failure |
| 4 | No projects detected |

## Configuration

`compiler` loads configuration from `.compiler\config.toml` in the project root, falling back to `%APPDATA%\compiler\config.toml`:

```toml
[ts]
runtime = "bun"
package_manager = "bun"

[python]
runner = "uv"

[c]
compiler = "clang"
build_system = "cmake"

[rust]
linker = "default"

[lint]
ts = "biome"
python = "ruff"
rust = "clippy"

[ai]
provider = "ollama"
model = "llama3"
endpoint = "http://127.0.0.1:11434"
```

The AI provider can be `ollama` (default, local), `anthropic` (requires `ANTHROPIC_API_KEY`), or any OpenAI-compatible endpoint (requires `OPENAI_API_KEY`).

Run `compiler init` to auto-generate `.compiler\config.toml` based on detected projects. Unknown config sections and invalid provider names will produce warnings. Build caches are stored in `.compiler\cache\`. Add `.compiler/` to your `.gitignore`.

## Architecture

```
src/
  main.rs           CLI entry point and command dispatch
  cli.rs            Argument parsing via clap derive
  config.rs         TOML config loading, generation, and validation
  types.rs          Core traits (Plugin) and shared types
  walker.rs         Recursive project discovery and watch mode (notify + polling fallback)
  orchestrator.rs   Dependency resolution, parallel execution, caching, result reporting
  ai.rs             LLM integration with batched-by-file fixing and rollback logic
  plugins/
    mod.rs          Plugin registry
    rust.rs         Rust plugin (cargo)
    go.rs           Go plugin (go build)
    typescript.rs   TypeScript plugin (bun/node/deno)
    c.rs            C/C++ plugin (cmake/make + clang/gcc)
    python.rs       Python plugin (uv/pip/poetry/pdm)
    zig.rs          Zig plugin (zig build)
```

The orchestrator builds a dependency graph from manifest analysis, groups projects by topological level for maximum parallelism, and dispatches each level to Rayon's thread pool. Successful builds are cached by content hash; unchanged projects are skipped on subsequent runs. Each plugin encapsulates detection, build, lint, and clean operations for its language. The AI fixer batches all errors per-file into a single LLM prompt, validates response quality, and rolls back suspicious fixes.

## Development

```powershell
cargo build
cargo test
cargo clippy
```

Requires Rust 1.85+ (edition 2024). Key dependencies: clap, clap_complete, rayon, reqwest (blocking, rustls-tls), serde, toml, walkdir, notify, regex, colored, dirs, sha2.

## License

MIT License
