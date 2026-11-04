# 编一份「青简 Dev」并注册成第二条输入法，与正式版同时存在。
#
# 两套的 CLSID / profile / 管道 / 互斥体 / 数据目录都不同（见 crates/qingjian-platform/src/identity.rs），
# 所以可以同时注册、同时运行，随时在任务栏切过去试新代码，不影响日常用的正式版。
#
# 用法（管理员）：
#   pwsh -File apps\windows\scripts\dev-variant.ps1              # 编 + 装 + 注册 + 起
#   pwsh -File apps\windows\scripts\dev-variant.ps1 -SkipBuild   # 只装 + 注册 + 起
#   pwsh -File apps\windows\scripts\dev-variant.ps1 -Unregister   # 反注册（保留目录）
#
# 装的位置缺省是 %LOCALAPPDATA%\QingjianDev，数据在 %APPDATA%\Qingjian-dev，与正式版各一份。

[CmdletBinding()]
param(
    # 已安装的正式版目录，dev 版从这里拷随包数据（词库 / 释义表 / 语言模型）。
    [string]$InstallDir = "$env:ProgramFiles\Qingjian",

    # dev 版装到哪。
    [string]$DevDir = "$env:LOCALAPPDATA\QingjianDev",

    [switch]$SkipBuild,
    [switch]$Unregister
)

$ErrorActionPreference = 'Stop'

# apps\windows\scripts -> apps\windows -> apps -> 仓库根
$repo = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$clsid = '{7C3E5F1A-2B4D-4E8F-9A6C-1D2E3F4A5B6C}'
$dll = Join-Path $DevDir 'qingjian_tsf.dll'
$server = Join-Path $DevDir 'qingjian-server.exe'

function Assert-Path($path, $what) {
    if (-not (Test-Path -LiteralPath $path)) { throw "$what 不在：$path" }
}

$elevated = [Security.Principal.WindowsPrincipal]::new(
    [Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($Unregister) {
    if (-not $elevated) { throw '反注册要管理员（写 HKLM）' }
    if (Test-Path -LiteralPath $dll) {
        Write-Host '== 反注册「青简 Dev」' -ForegroundColor Cyan
        & regsvr32.exe /s /u $dll
        Start-Sleep -Milliseconds 500
    }
    Write-Host '== 停掉 dev 版 Server' -ForegroundColor Cyan
    Get-Process qingjian-server -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -like "$DevDir*" } |
        ForEach-Object { "   停 pid=$($_.Id)"; Stop-Process -Id $_.Id -Force }
    $still = (Get-ItemProperty "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Classes\CLSID\$clsid\InprocServer32" -ErrorAction SilentlyContinue).'(default)'
    if ($still) { Write-Host "   ！CLSID 还在：$still" -ForegroundColor Yellow } else { Write-Host '   已从注册表移除' }
    Write-Host ''
    Write-Host "目录与数据保留：$DevDir、$env:APPDATA\Qingjian-dev" -ForegroundColor DarkGray
    exit 0
}

Assert-Path $InstallDir '正式版安装目录（先装一次正式版，或者用 -InstallDir 指过去）'

if (-not $SkipBuild) {
    # 有进程加载着 dev 版 DLL 时编译写不进去，先点名（切走输入法就放开）
    $holders = @(tasklist /m qingjian_tsf.dll /NH 2>$null | Select-String -SimpleMatch 'qingjian_tsf.dll')
    Write-Host '== 编译「青简 Dev」' -ForegroundColor Cyan
    $env:QINGJIAN_UIACCESS = '0'
    $env:QINGJIAN_VARIANT = 'dev'
    Push-Location $repo
    try {
        cargo build -p qingjian-windows-server -p qingjian-windows-tsf
        if ($LASTEXITCODE -ne 0) { throw 'cargo build 失败' }
    }
    finally {
        Pop-Location
        Remove-Item Env:QINGJIAN_VARIANT -ErrorAction SilentlyContinue
    }
}
Assert-Path (Join-Path $repo 'target\debug\qingjian-server.exe') 'dev 版 Server 产物'
Assert-Path (Join-Path $repo 'target\debug\qingjian_tsf.dll') 'dev 版 DLL 产物'

Write-Host "== 装到 $DevDir" -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $DevDir | Out-Null
# 随包数据（词库 / 释义表 / 语言模型 / emoji / 方案）从正式版拷，dev 版不自己生成一份
foreach ($sub in 'data', 'assets', 'licenses') {
    $src = Join-Path $InstallDir $sub
    if (Test-Path -LiteralPath $src) {
        robocopy $src (Join-Path $DevDir $sub) /MIR /NFL /NDL /NJH /NJS | Out-Null
        if ($LASTEXITCODE -ge 8) { throw "robocopy $sub 失败（exit $LASTEXITCODE）" }
    }
}
# onnxruntime 等 exe 同级的运行库
Get-ChildItem -LiteralPath $InstallDir -Filter '*.dll' -File |
    Where-Object { $_.Name -notlike 'qingjian_tsf*' } |
    ForEach-Object { Copy-Item $_.FullName -Destination $DevDir -Force }

Copy-Item -LiteralPath (Join-Path $repo 'target\debug\qingjian-server.exe') -Destination $DevDir -Force
Copy-Item -LiteralPath (Join-Path $repo 'target\debug\qingjian_tsf.dll') -Destination $DevDir -Force
Write-Host '   Server、TSF DLL 与随包数据已就位'

if (-not $elevated) {
    Write-Host ''
    Write-Host '接下来要管理员，另开一个管理员 PowerShell 跑：' -ForegroundColor Yellow
    Write-Host "  regsvr32 `"$dll`"" -ForegroundColor Yellow
    exit 1
}

Write-Host '== 注册「青简 Dev」' -ForegroundColor Cyan
& regsvr32.exe /s $dll
Start-Sleep -Milliseconds 500
$registered = (
    Get-ItemProperty `
        -LiteralPath "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Classes\CLSID\$clsid\InprocServer32" `
        -ErrorAction SilentlyContinue
).'(default)'
if ($registered -ieq $dll) {
    Write-Host '   已注册（输入法列表里显示为「青简 Dev」）'
}
else {
    Write-Host "   ！注册可能没生效：InprocServer32 = «$registered»，期望 «$dll»" -ForegroundColor Yellow
}

Write-Host '== 起 dev 版 Server' -ForegroundColor Cyan
Get-Process qingjian-server -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "$DevDir*" } |
    ForEach-Object { Stop-Process -Id $_.Id -Force }
Start-Process -FilePath $server -WorkingDirectory $DevDir
Start-Sleep -Seconds 4

$devServer = Get-Process qingjian-server -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "$DevDir*" }
$releaseServer = Get-Process qingjian-server -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -notlike "$DevDir*" }

Write-Host ''
Write-Host '== 现在应该有两条输入法' -ForegroundColor Cyan
Write-Host "   dev 版 Server : $(if ($devServer) { "在跑 pid=$($devServer.Id)" } else { '没起来，看下面日志' })"
Write-Host "   正式版 Server: $(if ($releaseServer) { "在跑 pid=$($releaseServer.Id)" } else { '没在跑（切到正式版时由输入法自己拉起）' })"
Write-Host ''
Write-Host '   任务栏输入法切换器（或 Win+空格）里应能选到「青简 Dev」。'
Write-Host "   日志分开：$env:LOCALAPPDATA\Qingjian-dev\logs\"
Write-Host "   数据分开：$env:APPDATA\Qingjian-dev\"
Write-Host ''
Write-Host '   切回正式版：Win+空格 选「青简」；反注册 dev 版：' -ForegroundColor DarkGray
Write-Host "     pwsh -File $PSCommandPath -Unregister" -ForegroundColor DarkGray
