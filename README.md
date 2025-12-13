# passgen

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/KarnesTH/passgen)](https://github.com/KarnesTH/passgen/releases)
[![CI](https://github.com/KarnesTH/passgen/workflows/CI/badge.svg)](https://github.com/KarnesTH/passgen/actions)

A CLI tool to generate secure and random passwords.

## Features

- Generate secure passwords with customizable length (8-64 characters)
- Generate multiple passwords at once
- Save passwords to files with timestamps
- Cryptographically secure random number generation
- Password validation ensuring lowercase, uppercase, digits, and special characters

## Installation

### Using install script (Linux/macOS)

```bash
curl -LsSf https://raw.githubusercontent.com/KarnesTH/passgen/main/install.sh | sh
```

### Using install script (Windows PowerShell)

```powershell
irm https://raw.githubusercontent.com/KarnesTH/passgen/main/install.ps1 | iex
```

### Manual installation

Download the latest binary from [GitHub Releases](https://github.com/KarnesTH/passgen/releases) for your platform.

## Usage

### Generate a single password (default length: 16)

```bash
passgen
```

### Generate a password with custom length

```bash
passgen -l 20
```

### Generate multiple passwords

```bash
passgen -c 5
```

### Save password to file

```bash
passgen -s
```

### Save password to custom file

```bash
passgen -s -o mypasswords.txt
```

### Combine options

```bash
passgen -l 24 -c 3 -s -o passwords.txt
```

## Options

- `-l, --length <LENGTH>` - Length of the password (default: 16, range: 8-64)
- `-c, --count <COUNT>` - Number of passwords to generate (default: 1)
- `-s, --save` - Save passwords to a file
- `-o, --output <OUTPUT>` - Output filename when saving (default: passgen.txt)

## Examples

```bash
# Generate a 20-character password
passgen -l 20

# Generate 5 passwords of length 16
passgen -c 5

# Generate and save 3 passwords of length 24
passgen -l 24 -c 3 -s -o mypasswords.txt
```

## Building from source

```bash
git clone https://github.com/KarnesTH/passgen.git
cd passgen
cargo build --release
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Author

KarnesTH <p_haehnel@hotmail.de>

