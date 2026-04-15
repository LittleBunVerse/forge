param(
    [string]$Version = $(if ($env:FORGE_VERSION) { $env:FORGE_VERSION } else { "latest" }),
    [string]$InstallDir = $(if ($env:FORGE_INSTALL_DIR) { $env:FORGE_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Programs/forge/bin" })
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repo = "LittleBunVerse/forge"
$binaryName = "forge.exe"

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
switch ($arch) {
    "X64" { $target = "x86_64-pc-windows-msvc" }
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
        Write-Host "当前会话可先执行："
        Write-Host ('$env:PATH = "{0};" + $env:PATH' -f $InstallDir)
        Write-Host "然后运行："
        Write-Host "forge"
    }
}
finally {
    if (Test-Path $tempRoot) {
        Remove-Item -Path $tempRoot -Recurse -Force
    }
}
