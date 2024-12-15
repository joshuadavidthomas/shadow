# shadow

A CLI tool to manage shell aliases.

## Requirements

`shdw` is written with cross-platform compatibility in mind, aiming to support all major operating systems (Linux, macOS, Windows). It's only requirement at the moment is the Rust toolchain for installation.

## Installation

Install directly from the repository:

```bash
cargo install --git https://github.com/joshuadavidthomas/shadow
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
