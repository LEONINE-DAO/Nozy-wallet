

set -e

VERSION=${1:-"0.2.0"}
PLATFORM=${2:-"all"}

echo "🚀 Building NozyWallet Release v${VERSION}"
echo "Platform: ${PLATFORM}"


RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' 


RELEASE_DIR="releases/v${VERSION}"
mkdir -p "${RELEASE_DIR}"


build_cli() {
    local target=$1
    local binary_name=$2
    
    echo -e "${GREEN}Building CLI for ${target}...${NC}"
    
    cargo build --release --target "${target}" --bin nozy
    

    if [[ "$target" == *"windows"* ]]; then
        cp "target/${target}/release/${binary_name}" "${RELEASE_DIR}/nozy-${target}.exe"
    else
        cp "target/${target}/release/${binary_name}" "${RELEASE_DIR}/nozy-${target}"
    fi
    
  
    if [[ "$target" == *"windows"* ]]; then
        certutil -hashfile "${RELEASE_DIR}/nozy-${target}.exe" SHA256 > "${RELEASE_DIR}/nozy-${target}.exe.sha256" || true
    else
        shasum -a 256 "${RELEASE_DIR}/nozy-${target}" > "${RELEASE_DIR}/nozy-${target}.sha256"
    fi
    
    echo -e "${GREEN}✓ Built ${target}${NC}"
}


build_desktop() {
    echo -e "${GREEN}Building desktop app...${NC}"
    
    
    cd frontend
    npm install
    cd ..
    
    
    cargo tauri build --bundles all
    
    
    cp -r src-tauri/target/release/bundle/* "${RELEASE_DIR}/" || true
    
    echo -e "${GREEN}✓ Built desktop installers${NC}"
}


case "${PLATFORM}" in
    "linux")
        build_cli "x86_64-unknown-linux-gnu" "nozy"
        ;;
    "windows")
        build_cli "x86_64-pc-windows-msvc" "nozy.exe"
        ;;
    "macos")
        build_cli "x86_64-apple-darwin" "nozy"
        build_cli "aarch64-apple-darwin" "nozy"
        ;;
    "desktop")
        build_desktop
        ;;
    "all")
        echo -e "${YELLOW}Building all platforms...${NC}"
        build_cli "x86_64-unknown-linux-gnu" "nozy"
        build_cli "x86_64-pc-windows-msvc" "nozy.exe"
        build_cli "x86_64-apple-darwin" "nozy"
        build_cli "aarch64-apple-darwin" "nozy"
        build_desktop
        ;;
    *)
        echo -e "${RED}Unknown platform: ${PLATFORM}${NC}"
        echo "Usage: $0 [version] [platform]"
        echo "Platforms: linux, windows, macos, desktop, all"
        exit 1
        ;;
esac

echo -e "${GREEN}Generating hashes file...${NC}"
echo "# NozyWallet v${VERSION} - SHA256 Hashes" > "${RELEASE_DIR}/HASHES.txt"
echo "" >> "${RELEASE_DIR}/HASHES.txt"
echo "Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")" >> "${RELEASE_DIR}/HASHES.txt"
echo "" >> "${RELEASE_DIR}/HASHES.txt"

find "${RELEASE_DIR}" -name "*.sha256" -exec cat {} \; >> "${RELEASE_DIR}/HASHES.txt" || true

echo -e "${GREEN}✓ Release build complete!${NC}"
echo -e "${GREEN}Files are in: ${RELEASE_DIR}${NC}"

