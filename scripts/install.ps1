param(
    [string]$Version = $(if ($env:FORGE_VERSION) { $env:FORGE_VERSION } else { "latest" }),
    [string]$InstallDir = $(if ($env:FORGE_INSTALL_DIR) { $env:FORGE_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Programs/forge/bin" })
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# 检查执行策略 —— 如果脚本被管道调用（irm | iex）则不需要检查
$policy = Get-ExecutionPolicy -Scope CurrentUser
if ($policy -eq "Restricted" -or $policy -eq "AllSigned") {
    Write-Host ""
    Write-Host "当前 PowerShell 执行策略为 '$policy'，可能阻止脚本运行。" -ForegroundColor Yellow
    Write-Host "如果遇到错误，请先以管理员身份运行以下命令，然后重试：" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser" -ForegroundColor Cyan
    Write-Host ""
}

$repo = "LittleBunVerse/forge"
$binaryName = "forge.exe"

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
switch ($arch) {
    "X64" { $target = "x86_64-pc-windows-msvc" }
    "Arm64" { $target = "aarch64-pc-windows-msvc" }
    default {
        throw "当前 Windows 架构暂不支持预编译安装：$arch。请改用 cargo install --git https://github.com/$repo.git forge"
    }
}

$archiveName = "forge-$target.zip"
$downloadUrl = if ($Version -eq "latest") {
    "https://github.com/$repo/releases/latest/download/$archiveName"
} else {
    "https://github.com/$repo/releases/download/$Version/$archiveName"
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("forge-install-" + [System.Guid]::NewGuid().ToString("N"))
$archivePath = Join-Path $tempRoot $archiveName
$packageDir = Join-Path $tempRoot ("forge-$target")

try {
    New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

    Write-Host "正在下载 $downloadUrl"
    Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath

    Expand-Archive -Path $archivePath -DestinationPath $tempRoot -Force
    Copy-Item (Join-Path $packageDir $binaryName) (Join-Path $InstallDir $binaryName) -Force

    Write-Host ""
    Write-Host "Forge 已安装到：$(Join-Path $InstallDir $binaryName)"

    $pathEntries = $env:PATH -split ';'
    if ($pathEntries -contains $InstallDir) {
        Write-Host "现在可以直接运行：forge"
    } else {
        Write-Host "当前 PATH 里还没有 $InstallDir"
        Write-Host ""
        Write-Host "[一次性生效] 在当前 PowerShell 中执行："
        Write-Host ""
        Write-Host ('  $env:PATH = "{0};" + $env:PATH' -f $InstallDir)
        Write-Host ""
        Write-Host "[永久生效] 将安装目录写入用户级环境变量（仅需执行一次）："
        Write-Host ""
        Write-Host ('  [Environment]::SetEnvironmentVariable("PATH", "{0};" + [Environment]::GetEnvironmentVariable("PATH", "User"), "User")' -f $InstallDir)
        Write-Host ""
        Write-Host "然后重新打开 PowerShell，运行：forge"
    }
}
finally {
    if (Test-Path $tempRoot) {
        Remove-Item -Path $tempRoot -Recurse -Force
    }
}
