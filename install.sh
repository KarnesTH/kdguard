#!/usr/bin/env bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# GitHub repository
REPO="KarnesTH/kdguard"
BINARY_NAME="kdguard"

# Detect OS and architecture
detect_platform() {
    local os=""
    local arch=""
    
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        *)          echo -e "${RED}Error: Unsupported operating system${NC}" >&2; exit 1 ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="aarch64" ;;
        *)              echo -e "${RED}Error: Unsupported architecture${NC}" >&2; exit 1 ;;
    esac
    
    # macOS aarch64 needs special handling
    if [ "$os" = "macos" ] && [ "$arch" = "aarch64" ]; then
        echo "macos-aarch64"
    elif [ "$os" = "macos" ] && [ "$arch" = "x86_64" ]; then
        echo "macos-x86_64"
    elif [ "$os" = "linux" ] && [ "$arch" = "x86_64" ]; then
        echo "linux-x86_64"
    else
        echo -e "${RED}Error: Unsupported platform combination${NC}" >&2
        exit 1
    fi
}

# Get latest release tag
get_latest_tag() {
    local tag=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$tag" ]; then
        echo -e "${RED}Error: Failed to get latest version${NC}" >&2
        exit 1
    fi
    echo "$tag"
}

# Extract version from tag (remove 'v' prefix)
get_version_from_tag() {
    local tag="$1"
    echo "${tag#v}"
}

# Download and install
main() {
    local platform=$(detect_platform)
    local tag=$(get_latest_tag)
    local version=$(get_version_from_tag "$tag")
    
    echo -e "${GREEN}Installing kdguard ${tag} for ${platform}...${NC}"
    
    # Download URL with version number in filename
    local download_url="https://github.com/${REPO}/releases/download/${tag}/kdguard_${version}-${platform}"
    if [ "$platform" = "windows-x86_64" ]; then
        download_url="https://github.com/${REPO}/releases/download/${tag}/kdguard_${version}-${platform}.exe"
    fi
    
    # Create install directory
    local install_dir="${HOME}/.local/bin"
    mkdir -p "$install_dir"
    
    # Download binary
    echo -e "${YELLOW}Downloading from ${download_url}...${NC}"
    if ! curl -fsSL "$download_url" -o "${install_dir}/${BINARY_NAME}"; then
        echo -e "${RED}Error: Failed to download binary${NC}" >&2
        exit 1
    fi
    
    # Make executable
    chmod +x "${install_dir}/${BINARY_NAME}"
    
    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":${install_dir}:"* ]]; then
        echo -e "${YELLOW}Warning: ${install_dir} is not in your PATH${NC}"
        echo -e "${YELLOW}Add this line to your shell profile (.bashrc, .zshrc, etc.):${NC}"
        echo -e "${GREEN}export PATH=\"\${HOME}/.local/bin:\${PATH}\"${NC}"
    fi
    
    echo -e "${GREEN}Successfully installed kdguard to ${install_dir}/${BINARY_NAME}${NC}"
    echo -e "${GREEN}Run 'kdguard --help' to get started${NC}"
}

main "$@"

