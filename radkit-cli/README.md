# Radkit CLI

The official Command Line Interface for [Radkit](../README.md), designed to streamline the development of AI agents.

## Features

- **Project Scaffolding**: Create new agent projects from various templates.
- **Interactive TUI**: A terminal-based user interface for managing your projects.
- **Component Management**: Easily add, list, and remove tools and skills.
- **Provider Configuration**: Manage LLM providers (OpenAI, Anthropic, Gemini, etc.).
- **Build & Run**: Integrated commands to build and run your agents.

## Installation

```bash
cargo install radkit-cli
```

Or build from source:

```bash
git clone https://github.com/yourusername/radkit.git
cd radkit/radkit-cli
cargo install --path .
```

## Usage

### Creating a New Project

Create a new agent project with the `create` command:

```bash
radkit create my-agent
```

Options:
- `--template <name>`: Specify a template (default: `simple-agent`).
- `--provider <name>`: Specify the default LLM provider.
- `--path <path>`: Link to a local `radkit` dependency.

### Interactive TUI

Launch the interactive dashboard:

```bash
radkit ui
```

This opens a rich terminal interface where you can manage your project, view status, and perform actions visually.

### Managing Tools & Skills

Radkit CLI helps you manage the capabilities of your agent.

**Tools:**

```bash
radkit tool add <name>    # Add a new tool
radkit tool list          # List installed tools
radkit tool remove <name> # Remove a tool
```

**Skills:**

```bash
radkit skill add <name>    # Add a new skill
radkit skill list          # List installed skills
radkit skill remove <name> # Remove a skill
```

### Managing Providers

Configure which LLM providers your agent uses:

```bash
radkit provider add <name>
radkit provider list
radkit provider remove
```

### Building and Running

```bash
radkit build    # Build the project
radkit run      # Run the agent
```

## Templates

Run `radkit list` to see available templates. Common templates include:
- `simple-agent`: A basic starting point.
- `interactive-agent`: An agent designed for user interaction.
- `advanced-agent`: Includes more complex patterns.
- `rag-agent`: Setup for Retrieval Augmented Generation.

## License

MIT
