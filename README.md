# shadow

A CLI tool to manage your shell aliases through symlinks.

## Requirements

- Rust toolchain
  - The latest is recommended, but any recent version should hopefully work
- Written with cross-platform compatibility in mind, aiming to support all major operating systems (Linux, macOS, Windows).

## Installation

Install directly from the repository:

```bash
cargo install --git https://git.joshthomas.dev/josh/shadow
```

## Usage

### Adding a shadow

Create a new alias by shadowing an existing command:

```bash
shadow add ls exa  # 'ls' will now execute 'exa'
```

You can specify a custom installation directory:

```bash
shadow add --bin-path ~/.local/bin ls exa
```

### Removing a shadow

Remove an existing shadow to restore the original command:

```bash
shadow remove ls
```

### Listing shadows

View all active shadows:

```bash
shadow list
```
