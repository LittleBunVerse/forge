#!/bin/sh
set -eu

REPO="LittleBunVerse/forge"
BIN_NAME="forge"
INSTALL_DIR="${FORGE_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${FORGE_VERSION:-latest}"

usage() {
  cat <<'EOF'
用法: install.sh [-b <bin_dir>] [-v <version>]

选项：
  -b, --bin-dir   安装目录，默认 ~/.local/bin
  -v, --version   发布版本，默认 latest
  -h, --help      显示帮助

环境变量：
  FORGE_INSTALL_DIR   默认安装目录
  FORGE_VERSION       默认安装版本
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    -b|--bin-dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    -v|--version)
      VERSION="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "未知参数：$1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

download() {
  url="$1"
  output="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$output"
    return
  fi
  if command -v wget >/dev/null 2>&1; then
    wget -qO "$output" "$url"
    return
  fi
  echo "缺少下载工具，请先安装 curl 或 wget" >&2
  exit 1
}

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Darwin)
    platform="apple-darwin"
    ;;
  Linux)
    platform="unknown-linux-gnu"
    ;;
  *)
    echo "当前系统暂不支持预编译安装：$os" >&2
    echo "请改用：cargo install --git https://github.com/$REPO.git forge" >&2
    exit 1
    ;;
esac

case "$arch" in
  x86_64|amd64)
    cpu="x86_64"
    ;;
  arm64|aarch64)
    cpu="aarch64"
    ;;
  *)
    echo "当前架构暂不支持预编译安装：$arch" >&2
    echo "请改用：cargo install --git https://github.com/$REPO.git forge" >&2
    exit 1
    ;;
esac

target="$cpu-$platform"
archive_name="forge-$target.tar.gz"

if [ "$target" = "aarch64-unknown-linux-gnu" ]; then
  echo "Linux ARM64 暂未提供预编译安装包，请改用：" >&2
  echo "cargo install --git https://github.com/$REPO.git forge" >&2
  exit 1
fi

if [ "$VERSION" = "latest" ]; then
  download_url="https://github.com/$REPO/releases/latest/download/$archive_name"
else
  download_url="https://github.com/$REPO/releases/download/$VERSION/$archive_name"
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

mkdir -p "$INSTALL_DIR"

archive_path="$tmp_dir/$archive_name"
package_dir="$tmp_dir/forge-$target"

echo "正在下载 $download_url"
download "$download_url" "$archive_path"

tar -xzf "$archive_path" -C "$tmp_dir"
install -m 755 "$package_dir/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

echo
echo "Forge 已安装到：$INSTALL_DIR/$BIN_NAME"

case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    echo "现在可以直接运行：forge"
    ;;
  *)
    echo "当前 PATH 里还没有 $INSTALL_DIR"
    echo "请先执行："
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    echo "然后运行："
    echo "  forge"
    ;;
esac
