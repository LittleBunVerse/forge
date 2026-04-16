#!/bin/sh
set -eu

BIN_NAME="forge"
LEGACY_APP_NAME="aidev"
INSTALL_DIR="${FORGE_INSTALL_DIR:-$HOME/.local/bin}"
PURGE=0
ASSUME_YES=0

usage() {
  cat <<'EOF'
用法: uninstall.sh [选项]

选项：
  -b, --bin-dir <dir>   二进制安装目录，默认 ~/.local/bin
  --purge               同时删除配置目录（forge 和 legacy aidev）
  -y, --yes             跳过确认（非交互环境必需）
  -h, --help            显示帮助

说明：
  默认只删除 forge 二进制，保留配置（类似大多数软件的卸载）。
  加 --purge 可同时删除 ~/.config/forge 和 ~/.config/aidev 配置目录。

环境变量：
  FORGE_INSTALL_DIR   二进制安装目录
  FORGE_CONFIG_DIR    配置目录基址（优先级高于 XDG_CONFIG_HOME）
  XDG_CONFIG_HOME     标准 XDG 配置根目录
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    -b|--bin-dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    --purge)
      PURGE=1
      shift
      ;;
    -y|--yes)
      ASSUME_YES=1
      shift
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

# 解析配置目录基址（与 src/config.rs 的 base_config_dir 逻辑一致）
resolve_config_base() {
  if [ -n "${FORGE_CONFIG_DIR:-}" ]; then
    echo "$FORGE_CONFIG_DIR"
    return
  fi
  if [ -n "${XDG_CONFIG_HOME:-}" ]; then
    echo "$XDG_CONFIG_HOME"
    return
  fi
  echo "$HOME/.config"
}

resolve_legacy_config_base() {
  if [ -n "${AIDEV_CONFIG_DIR:-}" ]; then
    echo "$AIDEV_CONFIG_DIR"
    return
  fi
  if [ -n "${XDG_CONFIG_HOME:-}" ]; then
    echo "$XDG_CONFIG_HOME"
    return
  fi
  echo "$HOME/.config"
}

BIN_PATH="$INSTALL_DIR/$BIN_NAME"
CONFIG_BASE="$(resolve_config_base)"
LEGACY_BASE="$(resolve_legacy_config_base)"
CONFIG_DIR="$CONFIG_BASE/$BIN_NAME"
LEGACY_DIR="$LEGACY_BASE/$LEGACY_APP_NAME"

# 列出计划删除项
echo "将执行的卸载操作："
echo
if [ -e "$BIN_PATH" ]; then
  echo "  [删除] $BIN_PATH"
else
  echo "  [跳过] $BIN_PATH （不存在）"
  # 尝试从 PATH 中兜底定位
  if command -v "$BIN_NAME" >/dev/null 2>&1; then
    found="$(command -v "$BIN_NAME")"
    echo "  [发现] PATH 中存在其他 ${BIN_NAME}：$found"
    echo "         未自动处理，请手动删除或重新指定 --bin-dir"
  fi
fi

if [ "$PURGE" -eq 1 ]; then
  if [ -d "$CONFIG_DIR" ]; then
    echo "  [删除] $CONFIG_DIR （配置目录）"
  else
    echo "  [跳过] $CONFIG_DIR （配置目录不存在）"
  fi
  if [ -d "$LEGACY_DIR" ]; then
    echo "  [删除] $LEGACY_DIR （legacy aidev 配置）"
  else
    echo "  [跳过] $LEGACY_DIR （legacy 配置不存在）"
  fi
else
  echo
  echo "  （保留配置目录，如需一并删除请加 --purge）"
fi
echo

# 交互确认
if [ "$ASSUME_YES" -ne 1 ]; then
  if [ ! -t 0 ]; then
    echo "当前非交互式终端，请加 --yes 确认卸载" >&2
    exit 1
  fi
  printf "确认卸载？[y/N] "
  read -r answer
  case "$answer" in
    y|Y|yes|YES) ;;
    *)
      echo "已取消。"
      exit 0
      ;;
  esac
fi

# 执行删除
if [ -e "$BIN_PATH" ]; then
  rm -f "$BIN_PATH"
  echo "已删除：$BIN_PATH"
fi

if [ "$PURGE" -eq 1 ]; then
  if [ -d "$CONFIG_DIR" ]; then
    rm -rf "$CONFIG_DIR"
    echo "已删除：$CONFIG_DIR"
  fi
  if [ -d "$LEGACY_DIR" ]; then
    rm -rf "$LEGACY_DIR"
    echo "已删除：$LEGACY_DIR"
  fi
fi

# 检测 PATH 残留（只提示，不改 rc 文件）
shell_name="$(basename "${SHELL:-/bin/sh}")"
case "$shell_name" in
  zsh)  profile_file="$HOME/.zshrc" ;;
  bash)
    if [ -f "$HOME/.bash_profile" ]; then
      profile_file="$HOME/.bash_profile"
    else
      profile_file="$HOME/.bashrc"
    fi
    ;;
  fish) profile_file="$HOME/.config/fish/config.fish" ;;
  *)    profile_file="$HOME/.profile" ;;
esac

if [ -f "$profile_file" ] && grep -qF "$INSTALL_DIR" "$profile_file" 2>/dev/null; then
  echo
  echo "提示：$profile_file 中仍包含 $INSTALL_DIR 相关的 PATH 配置。"
  echo "如果该目录只用于 forge，可手动编辑删除对应行："
  echo "  $profile_file"
fi

echo
echo "Forge 已卸载。"
