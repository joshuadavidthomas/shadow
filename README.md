# shdw

A CLI tool to manage your shell aliases.

## Requirements

- Rust toolchain
  - The latest is recommended, but any recent version should hopefully work
- Written with cross-platform compatibility in mind, aiming to support all major operating systems (Linux, macOS, Windows).

## Installation

Install directly from the repository:

```bash
cargo install --git https://github.com/joshuadavidthomas/shdw
```

## Usage

### Adding an alias

Create a new alias by shadowing an existing command:

```bash
shdw add ls exa  # 'ls' will now execute 'exa'
```

You can specify a custom installation directory:

```bash
shdw add --bin-path ~/.local/bin ls exa
```

### Removing an alias

Remove an existing alias to restore the original command:

```bash
shdw remove ls
```

### Listing aliases

View all active aliases:

```bash
shdw list
```
