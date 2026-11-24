# PowerShell install script for kdguard

$ErrorActionPreference = "Stop"

# GitHub repository
$REPO = "KarnesTH/kdguard"
$BINARY_NAME = "kdguard.exe"

# Colors for output
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

# Detect platform
function Get-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "AMD64") {
        return "windows-x86_64"
    } else {
        Write-ColorOutput Red "Error: Unsupported architecture: $arch"
        exit 1
    }
}

# Get latest release version
function Get-LatestVersion {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
        $tag = $response.tag_name
        # Remove 'v' prefix if present
        $version = $tag -replace '^v', ''
        return @{
            Tag = $tag
            Version = $version
        }
    } catch {
        Write-ColorOutput Red "Error: Failed to get latest version"
        exit 1
    }
}

# Main installation
function Main {
    $platform = Get-Platform
    $versionInfo = Get-LatestVersion
    $tag = $versionInfo.Tag
    $version = $versionInfo.Version
    
    Write-ColorOutput Green "Installing kdguard $tag for $platform..."
    
    # Download URL with version number in filename
    $downloadUrl = "https://github.com/$REPO/releases/download/$tag/kdguard_${version}-${platform}.exe"
    
    # Create install directory (using Windows standard LocalAppData)
    $installDir = Join-Path $env:LOCALAPPDATA "Karnes Development/kdguard"
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    }
    
    $installPath = Join-Path $installDir $BINARY_NAME
    
    # Download binary and checksum to a temporary directory
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("kdguard_" + [System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null
    $tmpBinary = Join-Path $tmpDir $BINARY_NAME
    $tmpChecksum = "$tmpBinary.sha256"

    try {
        Write-ColorOutput Yellow "Downloading from $downloadUrl..."
        try {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $tmpBinary -UseBasicParsing
        } catch {
            Write-ColorOutput Red "Error: Failed to download binary"
            exit 1
        }
        try {
            Invoke-WebRequest -Uri "$downloadUrl.sha256" -OutFile $tmpChecksum -UseBasicParsing
        } catch {
            Write-ColorOutput Red "Error: No checksum published for this release"
            exit 1
        }

        $expected = ((Get-Content $tmpChecksum -Raw).Trim() -split '\s+')[0].ToLower()
        $actual = (Get-FileHash -Algorithm SHA256 $tmpBinary).Hash.ToLower()
        if ($expected -ne $actual) {
            Write-ColorOutput Red "Error: Checksum does not match, installation canceled"
            exit 1
        }
        Write-ColorOutput Green "Checksum verified"

        Move-Item -Path $tmpBinary -Destination $installPath -Force
    } finally {
        Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }

    # Create symlink alias 'kdg.exe'
    $aliasName = "kdg.exe"
    $aliasPath = Join-Path $installDir $aliasName
    if (Test-Path $aliasPath) {
        Remove-Item $aliasPath -Force
    }
    try {
        New-Item -ItemType SymbolicLink -Path $aliasPath -Target $installPath -Force | Out-Null
    } catch {
        Copy-Item -Path $installPath -Destination $aliasPath -Force
    }

    # Check if install directory is in PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        Write-ColorOutput Yellow "Warning: $installDir is not in your PATH"
        Write-ColorOutput Yellow "Adding to PATH..."
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
        Write-ColorOutput Green "Added to PATH. Please restart your terminal for changes to take effect."
    }

    Write-ColorOutput Green "Successfully installed kdguard to $installPath"
    Write-ColorOutput Green "Run 'kdg' to get started"
}

Main