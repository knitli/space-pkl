# space-pklr

🌙 Pkl schema and template generation for [Moon](https://github.com/moonrepo/moon) workspace configurations.

[![Crates.io](https://img.shields.io/crates/v/space-pklr.svg)](https://crates.io/crates/space-pklr)
[![Documentation](https://docs.rs/space-pklr/badge.svg)](https://docs.rs/space-pklr)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`space-pklr` generates type-safe [Pkl](https://pkl-lang.org/) schemas from Moon's configuration types, enabling robust configuration authoring with IDE support, validation, and documentation.

## Features

- 🔧 **Complete Moon Support**: Generates schemas for all Moon configuration types
- 📝 **Type Safety**: Leverages Pkl's type system for configuration validation
- 🎯 **IDE Integration**: Full IntelliSense and autocomplete support
- 📚 **Documentation**: Automatic documentation generation from Moon's schema annotations
- ⚡ **CLI Tool**: Easy-to-use command-line interface
- 🔌 **Programmatic API**: Integrate into your own tools and workflows

## Installation

### From Crates.io

```bash
cargo install space-pklr
```

### From Source

```bash
git clone https://github.com/knitli/space-pklr
cd space-pklr
cargo install --path .
```

## Quick Start

### Generate All Schemas

```bash
space-pklr generate
```

This creates a `pkl-schemas/` directory with:

- `Workspace.pkl` - Workspace configuration schema
- `Project.pkl` - Project configuration schema
- `Template.pkl` - Template configuration schema
- `Toolchain.pkl` - Toolchain configuration schema
- `Tasks.pkl` - Tasks configuration schema
- `mod.pkl` - Module index

### Generate Specific Schema

```bash
# Generate only workspace schema
space-pklr generate workspace

# Generate with custom options
space-pklr generate workspace --no-comments --output ./schemas
```

### Use in Your Pkl Files

```pkl
// Workspace.pkl
import "pkl-schemas/Workspace.pkl"

config: workspace.WorkspaceConfig = new {
  projects = new {
    sources = new {
      "my-app" = "./apps/my-app"
      "shared-lib" = "./libs/shared"
    }
  }

  experiments = new {
    actionPipelineV2 = true
  }

  hasher = new {
    optimization = "Performance"
    walkStrategy = "Vcs"
  }
}
```

## CLI Reference

### Commands

#### `generate [TYPE]`

Generate Pkl schemas for Moon configurations.

**Arguments:**
- `TYPE` - Schema type to generate: `workspace`, `project`, `template`, `toolchain`, `tasks`, or `all` (default: `all`)

**Options:**
- `--output, -o <DIR>` - Output directory (default: `./pkl-schemas`)
- `--no-comments` - Exclude comments from generated schemas
- `--no-examples` - Exclude examples from generated schemas
- `--header <TEXT>` - Custom header text
- `--footer <TEXT>` - Custom footer text
- `--module-name <NAME>` - Module name for schemas (default: `moon`)
- `--single-file` - Generate as single file instead of split types

**Examples:**

```bash
# Generate all schemas with defaults
space-pklr generate

# Generate only workspace schema without comments
space-pklr generate workspace --no-comments

# Custom output directory and module name
space-pklr generate --output ./my-schemas --module-name myproject
```

#### `init <TYPE>`

Initialize a new Pkl configuration from templates.

```bash
space-pklr init workspace --output Workspace.pkl --with-examples
```

#### `validate <FILE>`

Validate an existing Pkl configuration.

```bash
space-pklr validate Workspace.pkl --config-type workspace
```

### Global Options

- `--log-level <LEVEL>` - Set logging level: `trace`, `debug`, `info`, `warn`, `error` (default: `info`)

## Programmatic API

### Basic Usage

```rust
use space_pklr::prelude::*;

fn main() -> space_pklr::Result<()> {
    // Generate workspace schema
    let schema = generate_workspace_schema()?;
    println!("{}", schema);

    // Or use the generator directly
    let config = GeneratorConfig::default();
    let generator = SchemaGenerator::new(config);
    generator.generate_all()?;

    Ok(())
}
```

### Custom Configuration

```rust
use space_pklr::prelude::*;
use std::path::PathBuf;

fn main() -> space_pklr::Result<()> {
    let config = GeneratorConfig {
        include_comments: true,
        include_examples: true,
        output_dir: PathBuf::from("./custom-schemas"),
        module_name: "myproject".to_string(),
        header: Some("// Custom header\n".to_string()),
        ..Default::default()
    };

    let generator = SchemaGenerator::new(config);

    // Generate specific schemas
    let workspace_schema = generator.generate_workspace_schema()?;
    let project_schema = generator.generate_project_schema()?;

    // Or generate all
    generator.generate_all()?;

    Ok(())
}
```

## Generated Schema Structure

The generated schemas follow Pkl conventions and include:

### Type Definitions

```pkl
/// Workspace configuration for Moon
class WorkspaceConfig {
  /// Configure code ownership rules for generating a CODEOWNERS file
  codeowners: (CodeownersConfig)?

  /// Configure boundaries and constraints between projects
  constraints: (ConstraintsConfig)?

  /// Configure all projects within the workspace
  projects: WorkspaceProjects

  // ... more fields
}
```

### Validation and Constraints

```pkl
class HasherConfig {
  /// The optimization to use when hashing
  optimization: ("Accuracy"|"Performance") = "Accuracy"

  /// File paths that match a configured glob pattern
  ignorePatterns: Listing<String>

  /// Logs a warning when a task has configured an explicit file path input
  warnOnMissingInputs: Boolean = true
}
```

### Documentation and Examples

All schemas include comprehensive documentation extracted from Moon's configuration types, plus generated examples showing proper usage.

## Moon Configuration Support

`space-pklr` supports all Moon configuration types:

| Configuration | Schema File | Description |
|---------------|-------------|-------------|
| Workspace | `Workspace.pkl` | Root workspace configuration |
| Project | `Project.pkl` | Individual project configuration |
| Template | `Template.pkl` | Code generation templates |
| Toolchain | `Toolchain.pkl` | Language toolchain settings |
| Tasks | `Tasks.pkl` | Shared task definitions |

## Development

### Prerequisites

- Rust 1.70+
- Moon (for testing with real configurations)

### Building

```bash
git clone https://github.com/knitli/space-pklr
cd space-pklr
cargo build --release
```

### Testing

```bash
cargo test
cargo test --features fancy-errors
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## Related Projects

- [Moon](https://github.com/moonrepo/moon) - Universal build tool and codebase management
- [Pkl](https://pkl-lang.org/) - Configuration programming language
- [schematic](https://github.com/moonrepo/schematic) - Schema-driven configuration management

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.
