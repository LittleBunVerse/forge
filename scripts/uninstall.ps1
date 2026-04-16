param(
    [string]$InstallDir = $(if ($env:FORGE_INSTALL_DIR) { $env:FORGE_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Programs/forge/bin" }),
    [switch]$Purge,
    [switch]$Yes
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$binaryName = "forge.exe"
$appName = "forge"
$legacyAppName = "aidev"

# 解析配置目录基址（与 src/config.rs 的 base_config_dir 一致：
# FORGE_CONFIG_DIR > XDG_CONFIG_HOME > $HOME/.config）
function Resolve-ConfigBase {
    param([string]$OverrideEnv)
    if ($OverrideEnv -and (Get-Item "env:$OverrideEnv" -ErrorAction SilentlyContinue)) {
        return (Get-Item "env:$OverrideEnv").Value
    }
    if ($env:XDG_CONFIG_HOME) { return $env:XDG_CONFIG_HOME }
    $home_ = if ($env:HOME) { $env:HOME } elseif ($env:USERPROFILE) { $env:USERPROFILE } else { throw "无法确定 HOME 目录" }
    return (Join-Path $home_ ".config")
}

$binPath = Join-Path $InstallDir $binaryName
$configBase = Resolve-ConfigBase -OverrideEnv "FORGE_CONFIG_DIR"
$legacyBase = Resolve-ConfigBase -OverrideEnv "AIDEV_CONFIG_DIR"
$configDir = Join-Path $configBase $appName
$legacyDir = Join-Path $legacyBase $legacyAppName

# 列出计划删除项
Write-Host "将执行的卸载操作："
Write-Host ""

if (Test-Path $binPath) {
    Write-Host "  [删除] $binPath"
} else {
    Write-Host "  [跳过] $binPath （不存在）"
    $foundCmd = Get-Command $appName -ErrorAction SilentlyContinue
    if ($foundCmd) {
        Write-Host "  [发现] PATH 中存在其他 forge：$($foundCmd.Source)"
        Write-Host "         未自动处理，请手动删除或通过 -InstallDir 指定正确目录"
    }
}

if ($Purge) {
    if (Test-Path $configDir) {
        Write-Host "  [删除] $configDir （配置目录）"
    } else {
        Write-Host "  [跳过] $configDir （配置目录不存在）"
    }
    if (Test-Path $legacyDir) {
        Write-Host "  [删除] $legacyDir （legacy aidev 配置）"
    } else {
        Write-Host "  [跳过] $legacyDir （legacy 配置不存在）"
    }
} else {
    Write-Host ""
    Write-Host "  （保留配置目录，如需一并删除请加 -Purge）"
}
Write-Host ""

# 交互确认
if (-not $Yes) {
    $answer = Read-Host "确认卸载？[y/N]"
    if ($answer -notmatch '^(y|Y|yes|YES)$') {
        Write-Host "已取消。"
        exit 0
    }
}

# 执行删除
if (Test-Path $binPath) {
    Remove-Item -Path $binPath -Force
    Write-Host "已删除：$binPath"
}

if ($Purge) {
    if (Test-Path $configDir) {
        Remove-Item -Path $configDir -Recurse -Force
        Write-Host "已删除：$configDir"
    }
    if (Test-Path $legacyDir) {
        Remove-Item -Path $legacyDir -Recurse -Force
        Write-Host "已删除：$legacyDir"
    }
}

# 检查用户级 PATH 是否还包含 InstallDir
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath) {
    $entries = $userPath -split ';' | Where-Object { $_ -ne '' }
    $hit = $entries | Where-Object { $_ -eq $InstallDir }
    if ($hit) {
        Write-Host ""
        Write-Host "检测到用户级 PATH 环境变量中仍包含：$InstallDir" -ForegroundColor Yellow
        if (-not $Yes) {
            $clean = Read-Host "是否从 PATH 中移除该条目？[y/N]"
        } else {
            $clean = "n"
        }
        if ($clean -match '^(y|Y|yes|YES)$') {
            $newEntries = $entries | Where-Object { $_ -ne $InstallDir }
            $newPath = $newEntries -join ';'
            [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
            Write-Host "已从用户级 PATH 中移除：$InstallDir"
            Write-Host "请重启终端让改动生效。"
        } else {
            Write-Host "已跳过 PATH 清理。如需手动处理，可在 PowerShell 中执行："
            Write-Host "  `$p = [Environment]::GetEnvironmentVariable('PATH','User')"
            Write-Host "  `$new = (`$p -split ';' | Where-Object { `$_ -ne '$InstallDir' }) -join ';'"
            Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `$new, 'User')"
        }
    }
}

Write-Host ""
Write-Host "Forge 已卸载。"
