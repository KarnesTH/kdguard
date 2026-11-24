#!/bin/sh
set -eu

REPO="KarnesTH/kdguard"
BINARY_NAME="kdguard"
ALIAS_NAME="kdg"
INSTALL_DIR="${HOME}/.local/bin"

if [ -t 1 ]; then
    RED=$(printf '\033[0;31m')
    GREEN=$(printf '\033[0;32m')
    YELLOW=$(printf '\033[1;33m')
    NC=$(printf '\033[0m')
else
    RED=""
    GREEN=""
    YELLOW=""
    NC=""
fi

fail() {
    printf '%sError: %s%s\n' "$RED" "$1" "$NC" >&2
    exit 1
}

detect_platform() {
    case "$(uname -s)" in
        Linux*) os="linux" ;;
        Darwin*) os="macos" ;;
        *) fail "Unsupported operating system" ;;
    esac

    case "$(uname -m)" in
        x86_64 | amd64) arch="x86_64" ;;
        arm64 | aarch64) arch="aarch64" ;;
        *) fail "Unsupported architecture" ;;
    esac

    case "${os}-${arch}" in
        linux-x86_64 | macos-x86_64 | macos-aarch64) echo "${os}-${arch}" ;;
        *) fail "No release for ${os}-${arch}" ;;
    esac
}

get_latest_tag() {
    tag=$(curl -fsSL --proto '=https' "https://api.github.com/repos/${REPO}/releases/latest" |
        sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([A-Za-z0-9.-]*\)".*/\1/p' |
        head -n 1)
    [ -n "$tag" ] || fail "Failed to get the latest version"
    echo "$tag"
}

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d ' ' -f 1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d ' ' -f 1
    else
        fail "Neither sha256sum nor shasum found"
    fi
}

main() {
    platform=$(detect_platform)
    tag=$(get_latest_tag)
    version="${tag#v}"
    asset="kdguard_${version}-${platform}"
    url="https://github.com/${REPO}/releases/download/${tag}/${asset}"

    printf '%sInstalling kdguard %s for %s%s\n' "$GREEN" "$tag" "$platform" "$NC"

    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT INT TERM

    printf '%sDownloading %s%s\n' "$YELLOW" "$url" "$NC"
    curl -fsSL --proto '=https' "$url" -o "${tmp_dir}/${asset}" || fail "Failed to download the binary"
    curl -fsSL --proto '=https' "${url}.sha256" -o "${tmp_dir}/${asset}.sha256" ||
        fail "No checksum published for this release"

    expected=$(cut -d ' ' -f 1 "${tmp_dir}/${asset}.sha256" | tr 'A-F' 'a-f')
    actual=$(sha256_of "${tmp_dir}/${asset}")
    [ "$expected" = "$actual" ] || fail "Checksum does not match, installation canceled"
    printf '%sChecksum verified%s\n' "$GREEN" "$NC"

    mkdir -p "$INSTALL_DIR"
    chmod 755 "${tmp_dir}/${asset}"
    mv -f "${tmp_dir}/${asset}" "${INSTALL_DIR}/${BINARY_NAME}"
    ln -sf "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${ALIAS_NAME}"

    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            printf '%sWarning: %s is not in your PATH%s\n' "$YELLOW" "$INSTALL_DIR" "$NC"
            printf '%sAdd this line to your shell profile (.bashrc, .zshrc, ...):%s\n' "$YELLOW" "$NC"
            printf '%sexport PATH="${HOME}/.local/bin:${PATH}"%s\n' "$GREEN" "$NC"
            ;;
    esac

    printf '%sInstalled kdguard to %s/%s%s\n' "$GREEN" "$INSTALL_DIR" "$BINARY_NAME" "$NC"
    printf '%sRun kdg to get started%s\n' "$GREEN" "$NC"
}

main "$@"