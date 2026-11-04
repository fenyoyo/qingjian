# 正式版与开发版同时装在一台机器上

调试 Windows 壳时过去只有一条路：把 `target\debug` 的 Server 与 DLL 换到已安装目录上跑。问题是两边的
CLSID、命名管道、单实例互斥体是同一套，**同一时刻只能活一个**——切过去没反应、改了代码像没生效、
装着装着输入法从列表里消失了，这类「静默二选一」很难查。

现在两套可以同时存在：正式版照常日常用，开发版随时换成新代码，任务栏切一下就能对比。

## 怎么用

管理员 PowerShell：

```powershell
# 编 dev 版、装到 %LOCALAPPDATA%\QingjianDev、注册成「青简 Dev」并起它
pwsh -File apps\windows\scripts\dev-variant.ps1

# 已经编好了，只装 + 注册 + 起
pwsh -File apps\windows\scripts\dev-variant.ps1 -SkipBuild

# 反注册（保留目录与数据）
pwsh -File apps\windows\scripts\dev-variant.ps1 -Unregister
```

装完任务栏输入法切换器（或 Win+空格）里会有两条：**青简** 与 **青简 Dev**，随便切。

## 两套到底分开了什么

| 标识 | 正式版 | 开发版 |
| --- | --- | --- |
| CLSID | `{4FDCA82D-…}` | `{7C3E5F1A-…}` |
| TSF 语言 profile | `{8119F8E0-…}` | `{9D4F2A1B-…}` |
| 显示名 | 青简 | 青简 Dev |
| 命名管道 | `\\.\pipe\qingjian` | `\\.\pipe\qingjian-dev` |
| Server 单实例互斥体 | `Local\QingjianServer` | `Local\QingjianServer-dev` |
| 拉起互斥体 | `Local\QingjianServerLaunch` | `Local\QingjianServerLaunch-dev` |
| 数据目录 | `%APPDATA%\Qingjian` | `%APPDATA%\Qingjian-dev` |
| 日志目录 | `%LOCALAPPDATA%\Qingjian\logs` | `%LOCALAPPDATA%\Qingjian-dev\logs` |
| 图标文件 | `%ProgramData%\Qingjian\qingjian.ico` | `%ProgramData%\Qingjian\qingjian-dev.ico` |

取值全部在 `crates/qingjian-platform/src/identity.rs`。

**数据目录分开是必要的**，不只是整洁：两个 Server 会各自持有 `FrequencyLearner` 并按自己的节奏落盘
`user.tsv` / `user-ngram.tsv`，共用一份就是互相覆盖学习数据。

`INSTALLER_MUTEX`（`Global\QingjianInstaller`）是唯一故意共用的：装正式版时安装程序要替换 exe，
这时开发版的 DLL 也不该去拉 Server 占住文件。

## 手动编 dev 版

脚本里做的事，手敲就一步——**设了环境变量再 `cargo build`**：

```powershell
$env:QINGJIAN_VARIANT = 'dev'
cargo build -p qingjian-windows-server -p qingjian-windows-tsf
```

产物在 `target\debug\`，与正式版构建**互相覆盖**（同一个 target 目录）。所以：
编完 dev 就直接跑脚本 `-SkipBuild` 装走；之后再编正式版，记得把环境变量去掉，
否则会继续出 dev 版——验证办法是 `strings` 一下 DLL 里有没有 `qingjian-dev`。

## 实现上的两个约束

**cfg 不跨 crate。** build.rs 发的是 `cargo:rustc-cfg=qingjian_variant_dev`，rustc 的 cfg 只对发出它的那个 crate 有效。
所以判别只在 `qingjian-platform` 内部做（`identity.rs` 里一对 `#[cfg]` 常量），
tsf / server 一律引用 `qingjian_platform::identity::*`。别在自己的 crate 里写那个 cfg，它永远为假。

**正式版那半边不能动。** 那是已发布产品的身份：CLSID 一换，升级后系统里的输入法条目就找不回原来的 DLL，
表现为「升级完青简从列表里消失了」。`identity.rs` 的 `release_identity_is_frozen` 测试把正式版的每个取值钉死，
改了就红；`dev_identity_differs_from_release` 则保证开发版每个值都与正式版错开。

## 排查

- **列表里只有一条**：确认 `reg query "HKLM\SOFTWARE\Classes\CLSID\{7C3E5F1A-2B4D-4E8F-9A6C-1D2E3F4A5B6C}\InprocServer32"` 有值，
  且 `HKLM\SOFTWARE\Microsoft\CTF\TIP\{7C3E5F1A-…}\LanguageProfile\0x00000804` 下的 `Description` 是「青简 Dev」。
- **切到青简 Dev 没反应**：看 `%LOCALAPPDATA%\Qingjian-dev\logs\`。没有 `server.*.log` 说明 Server 没起来；
  有日志但 `tsf.*.log` 里报连不上，多半是两个 Server 抢了同一个管道——确认 dev 版确实是用 `QINGJIAN_VARIANT=dev` 编的。
- **改了代码像没生效**：确认没有进程还挂着旧 DLL（`tasklist /m qingjian_tsf.dll`），
  切走输入法能放开；另外确认编完之后跑的是 `-SkipBuild`（否则脚本会再编一次，而这次不一定带着变量）。
