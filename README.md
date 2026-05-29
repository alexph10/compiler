#### compiler *win compatible
Drop into a directory containing Rust, Go, TypeScript, C/C++, Python, or Zig projects and `compiler` will discover them, resolve inter-project dependencies via topological sort, build independent projects in parallel, lint with your preferred toolchain, and optionally feed errors to an LLM for automated fixes with rollback safety.

#### Features

- **Polyglot Detection**: Walks the directory tree and identifies projects by manifest files (Cargo.toml, go.mod, package.json, CMakeLists.txt, Makefile)
- **Dependency-Aware Ordering**: Parses manifests for path/local dependencies and topologically sorts the build graph, with cycle detection fallback
- **Parallel Builds**: Independent projects within the dependency graph are built concurrently via Rayon using proper topological-level grouping
- **Build Caching**: Hashes source files with SHA-256 and skips rebuilds when nothing has changed
- **Unified Linting**: Delegates to per-language linters (Clippy, golangci-lint, Biome, clang-tidy, Ruff, zig build) through a common plugin interface
- **Watch Mode**: Uses `ReadDirectoryChangesW` via the `notify` crate with a polling fallback for automatic rebuild/lint on save
- **Plugin Architecture**: Each language is a `Plugin` trait implementation, making new languages a single-file addition
- **Configurable Toolchains**: Override runtimes, compilers, linkers, package managers, and linters via TOML config or CLI flags
- **JSON Output**: Machine-readable output for CI/CD integration
- **Shell Completions**: Auto-generated completions for PowerShell, Bash, Zsh, Fish, and Elvish
- **Config Validation**: Warns on unknown sections and invalid provider names in `.compiler/config.toml`
- **Dependency Graph Visualization**: ASCII or Graphviz DOT output of the project dependency graph
- **No System OpenSSL**: Ships with `rustls` so it runs as a standalone `compiler.exe` with no native crypto dependencies

#### Supported Languages

| Language | Manifest | Compiler/Runtime | Linter | Build System |
|---|---|---|---|---|
| Rust | `Cargo.toml` | cargo | clippy | cargo |
| Go | `go.mod` | go | golangci-lint | go build |
| TypeScript | `package.json` | bun/node/deno | biome/eslint | bun/npm/yarn/pnpm |
| C/C++ | `CMakeLists.txt`, `Makefile` | clang/gcc | clang-tidy | cmake/make |
| Python | `pyproject.toml` | uv/pip/poetry/pdm | ruff | uv/pip/poetry/pdm |
| Zig | `build.zig` | zig | zig build | zig build |

#### Install

```powershell
git clone https://github.com/alexph10/compiler.git
cd compiler
cargo build --release
Copy-Item .\target\release\compiler.exe "$env:USERPROFILE\.cargo\bin\"
```

Make sure `%USERPROFILE%\.cargo\bin` is on your `PATH`.

#### Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Build failure |
| 2 | Lint failure |
| 3 | AI fix failure |
| 4 | No projects detected |

#### Configuration

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

