# Changelog

## 0.7.0 (2026-11-24)

### Breaking

- Deterministic passwords are derived with PBKDF2-HMAC-SHA256 (600,000 iterations) and HKDF-SHA256 instead of HKDF only. **The same seed, salt and service give a different password than in 0.6.x.** Generate the new passwords with 0.7.0 and change them at the services.

### Added

- Terminal output with kd-ui: banner, sections, status lines, prompts, help and argument errors in the Karnes theme.
- Interactive mode: `kdg` without options asks for mode and values with the config values preselected.
- `kdg check` without argument asks for the password hidden, `kdg check -` reads it from stdin.
- `config edit` without options asks for every value, `-m, --mode` sets the default mode.
- `--no-save` as counterpart to `--save`.
- Mode follows from `--pattern`, `--words` or `--seed-env`/`--service`/`--salt` when `--mode` is missing.
- Without a terminal only the passwords are printed, one per line (`kdg | xclip`).
- Master seed is asked hidden and twice when `--seed-env` is missing; a weak seed is reported.
- Release workflow publishes a `.sha256` file per binary.
- `docs/cli.md` and `docs/security.md`.

### Changed

- Update check at most once a day, only in an interactive terminal, through `curl`. Failures are warnings instead of stopping the program.
- Updates, `install.sh` and `install.ps1` install a binary only when its SHA-256 matches the published checksum.
- Update replaces the running executable in place instead of a fixed install directory.
- `install.sh` is POSIX sh, so `curl … | sh` works with dash (Debian, Ubuntu).
- Translations are embedded and no longer written to the config directory; a new config takes the system language.
- Health check: length counts characters instead of bytes, exact matches of the common password list are rated weak, a password is no longer flagged for being part of a common password, other characters (e.g. umlauts) count for the entropy.
- Default passphrase length 6 words.
- Count is limited to 1 to 100.
- Config values are validated on load; an invalid file stops with the path and the reason.

### Fixed

- Modulo bias in the deterministic mode and in the random index.
- Saved password files are created with mode 0600 and no longer overwrite an existing file without confirmation.
- `auto_save` no longer writes a file when the output goes to a pipe.
- Platform checks compared `OS` with `"Windows"` and `"Darwin"`, so Windows used the Unix paths and macOS failed to create the log directory.
- Uninstall on Windows could delete the whole directory of the executable; only kdguard's own files are removed now.
- Invalid config files and missing translations panicked.
- `install.ps1` failed without permission to create symbolic links; it copies `kdg.exe` then.

### Removed

- Dependencies inquire, tokio, reqwest, sysinfo and lazy_static.
- `system_info.log` with hostname and hardware data.

## 0.6.5 (2025-12-20)

- Test release for the updater.

## 0.6.4 (2025-12-20)

- Update writes the new binary to a temporary file before replacing the old one.

## 0.6.3 (2025-12-20)

- Platform detection and update logic reworked.

## 0.6.2 (2025-12-20)

- Version bump to test the update.

## 0.6.1 (2025-12-20)

- Release tag is validated before updating.

## 0.6.0 (2025-12-20)

- Error handling with thiserror, logging with log and simplelog in all modules.
- New README badges.

## 0.5.4 (2025-12-19)

- Update command changed.

## 0.5.3 (2025-12-19)

- kdguard logo in the README.

## 0.5.2 (2025-12-19)

- Update command changed.

## 0.5.1 (2025-12-19)

- Language folder is created in the config directory.

## 0.5.0 (2025-12-19)

- Modes `pattern`, `phrase` (Diceware lists German and English) and `deterministic` (HKDF).
- Wordlists are cached.
- Unit tests for the health check.

## 0.4.0 (2025-12-18)

- Config file with `config show` and `config edit`.
- German and English with lingua-i18n-rs.
- Update and uninstall.

## 0.3.0 (2025-12-15)

- Project renamed to kdguard.

## 0.2.0 (2025-12-13)

- `check` command with password health check against the 10,000 most common passwords.
- Windows install path and PowerShell install command.

## 0.1.0 (2025-12-13)

- Random password generation with length and count.
- CI and release workflows, README, MIT license.