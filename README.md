# kdguard

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/KarnesTH/kdguard)](https://github.com/KarnesTH/kdguard/releases/latest)
[![CI](https://github.com/KarnesTH/kdguard/workflows/CI/badge.svg)](https://github.com/KarnesTH/kdguard/actions)

A CLI tool to generate secure and random passwords.

## Features

- Generate secure passwords with customizable length (8-64 characters)
- Generate multiple passwords at once
- Save passwords to files with timestamps
- Cryptographically secure random number generation
- Password validation ensuring lowercase, uppercase, digits, and special characters
- **Password Health Check** - Analyze password strength with score-based system
  - Length, character diversity, complexity, and entropy analysis
  - Detection of common passwords from 10k most-used passwords list
  - Pattern and repetition detection
  - Detailed warnings and improvement suggestions

## Installation

### Using install script (Linux/macOS)

```bash
curl -LsSf https://raw.githubusercontent.com/KarnesTH/kdguard/main/install.sh | sh
```

### Using install script (Windows PowerShell)

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/KarnesTH/kdguard/main/install.ps1 | iex"
```

### Manual installation

Download the latest binary from [GitHub Releases](https://github.com/KarnesTH/kdguard/releases) for your platform.

## Usage

### Generate a single password (default length: 16)

```bash
kdguard
```

### Generate a password with custom length

```bash
kdguard -l 20
```

### Generate multiple passwords

```bash
kdguard -c 5
```

### Save password to file

```bash
kdguard -s
```

### Save password to custom file

```bash
kdguard -s -o mypasswords.txt
```

### Combine options

```bash
kdguard -l 24 -c 3 -s -o passwords.txt
```

### Check password strength

```bash
kdguard check 'YourPassword123!'
```

### Check password with detailed analysis

```bash
kdguard check 'YourPassword123!' --detailed
```

**Note:** When checking passwords with special characters like `$`, `!`, `(`, `)`, use single quotes (`'`) to prevent shell interpretation.

## Options

### Generation Options

- `-l, --length <LENGTH>` - Length of the password (default: 16, range: 8-64)
- `-c, --count <COUNT>` - Number of passwords to generate (default: 1)
- `-s, --save` - Save passwords to a file
- `-o, --output <OUTPUT>` - Output filename when saving (default: kdguard.txt)

### Check Command Options

- `check <PASSWORD>` - Check password strength (use single quotes for special characters)
- `-d, --detailed` - Show detailed analysis with score breakdown, warnings, and suggestions

## Examples

```bash
# Generate a 20-character password
kdguard -l 20

# Generate 5 passwords of length 16
kdguard -c 5

# Generate and save 3 passwords of length 24
kdguard -l 24 -c 3 -s -o mypasswords.txt

# Check password strength
kdguard check 'MyPassword123!'

# Check password with detailed analysis
kdguard check 'MyPassword123!' --detailed

# Check password with special characters (use single quotes)
kdguard check '9$LyEq4#G+l3(P(O' --detailed
```

## Password Health Check

The `check` command analyzes passwords using a comprehensive scoring system:

- **Score Range:** 0-100 points
- **Rating Levels:**
  - Weak (0-40 points)
  - Medium (41-60 points)
  - Strong (61-80 points)
  - Very Strong (81-100 points)

### Scoring Categories

1. **Length Score (0-25 points):** Based on password length
2. **Character Diversity Score (0-30 points):** Checks for lowercase, uppercase, digits, and special characters
3. **Complexity Score (0-25 points):** Detects common patterns and repetitions
4. **Entropy Score (0-20 points):** Measures password entropy based on character set size

### Features

- Checks against 10,000 most common passwords
- Detects common patterns (sequences, keyboard patterns)
- Identifies character repetitions
- Provides actionable improvement suggestions

## Building from source

**Prerequisites:** You need to have [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
git clone https://github.com/KarnesTH/kdguard.git
cd kdguard
cargo build --release
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Author

Developed with ❤️ by KarnesTH

