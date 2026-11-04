# Windows 本机构建与联调的坑

在这台机器（Windows 10 19045 / MSVC BuildTools 14.44 / Rust 1.96）上第一次把 Windows 壳跑起来时踩到的问题，
按「症状 → 原因 → 绕法」记一遍，换机器时先看这里。

本文只写**已经证实**的结论；推断但没验过的会标出来。

## 1. `onig_sys` 的 C 编译在本会话里必然失败

`cargo build -p qingjian-windows-server` 会死在 `onig_sys`（candle → tokenizers 带进来的 oniguruma）。
两种错：

```
cl: 命令行 error D8050 :无法执行"…\c1.dll": 未能将命令行放入调试记录中
LINK : fatal error LNK1104: 无法打开文件"C:\Temp\lnk{432D65B2-…}.tmp"
```

第一条来自 `cl.exe` 编 `.c`，第二条来自 `lib.exe` 打包。**两条要分别绕，只解决一条还会死在另一条。**

### 绕法

```powershell
# 用 rustup 自带 LLVM 的 ar 代替 lib.exe（绕开 LNK1104）
rustup component add llvm-tools
$env:AR = "$env:RUSTUP_HOME\toolchains\<toolchain>\lib\rustlib\x86_64-pc-windows-msvc\bin\llvm-ar.exe"

# 关掉 debuginfo：cc-rs 就不给 cl 传 -Z7（绕开 D8050）
$env:CARGO_PROFILE_DEV_DEBUG = '0'

$env:QINGJIAN_UIACCESS = '0'   # 没签名的 exe 带 uiAccess 起不来（os error 740）
cargo build -p qingjian-windows-server -p qingjian-windows-tsf
```

这两条一起设才能过：**只设 `AR` 仍然会死在 D8050**（验过），因为 `-Z7` 是 debuginfo 带出来的，
cc-rs 不看 `AR`。反过来只设 `CARGO_PROFILE_DEV_DEBUG` 会死在 LNK1104。

### 排除掉的怀疑

排查时逐条验过，**都不是**原因，别再往这些方向找：

- **不是 vcvars 环境没设**：在 PowerShell 里把 `vcvars64.bat` 的全部环境变量回灌再调 `cl`，照样成功。
- **不是 `%TEMP%` 短名**：这台机器的 `TEMP` 是 `C:\Users\ADMINI~1\AppData\Local\Temp` 这种 8.3 短名，
  换成 `C:\Temp` 后错误形状不变。
- **不是 DSH 沙箱**：把同一条命令放进计划任务、以另一个会话跑，一样失败；而**手工**用同样的参数调 `cl.exe` / `lib.exe` 一次都没失败过。
- **不是内存不足**：`-j 1` 单线程一样失败。

### 结论的边界

目前能确定的是「`-Z7` 会让 `cl` 在这个环境里报 D8050」「`lib.exe` 在这个环境里报 LNK1104」，
**没有查明更底层的原因**（怀疑与进程创建调试记录 / 文件映射有关，未证实）。
所以上面给的是绕法，不是修复；换机器先直接试试不设这两个变量能不能过，能过就别绕。

## 2. Server 挂了不会自动拉起（管理员机器上尤其致命）

TSF DLL 连不上 Server 时会自己拉一个（`apps/windows/tsf/src/com/service/launch.rs`），
但它**只在宿主进程完整性级别为 Medium 时**才拉：

```
宿主进程不是普通桌面应用（完整性级别非 Medium），不拉起 Server
```

这是有意为之（LogonUI、UAC 安全桌面、AppContainer 里都不该拉），但代价是：
**以管理员身份登录、或整个会话以提升权限运行时，绝大多数进程都是 High**——
这台机器上 explorer、编辑器、浏览器宿主实测全是 0x3000（Medium 是 0x2000）。
这些进程里的青简一旦发现 Server 没了，只会反复刷日志、把字母放行，**永远等不到 Server 回来**。

现象是「切到青简，按字母没反应」或者「忽然只能打英文」。

### 排查

```powershell
Get-Process qingjian-server            # 不在 = 就是这个原因
Get-Content "$env:LOCALAPPDATA\Qingjian\logs\tsf.<日期>.log" -Tail 20   # 找上面那句
```

进程完整性级别看 `whoami /groups` 里的 `Mandatory Label\…` 那一行：

```
Mandatory Label\High Mandatory Level   Label   S-1-16-12288     ← 0x3000，Medium 是 S-1-16-8192
```

（中文系统里组名仍是英文 `Mandatory Label\…`，搜 `S-1-16` 最快。）

### 兜底

目前没有守护机制，Server 没了就手动起（它和 DLL 同目录）：

```powershell
Start-Process "D:\Application\Qingjian\qingjian-server.exe" -WorkingDirectory "D:\Application\Qingjian"
```

装在正式环境时由「启动」文件夹的快捷方式在登录时拉起，所以**注销重登**也能恢复。
日常调试建议加一条：改完代码先看一眼 `Get-Process qingjian-server` 还在不在。

## 3. 注册了 DLL ≠ 出现在输入法列表里

`regsvr32` 成功、注册表里 `CLSID\{4FDCA82D-…}\InprocServer32` 和
`CTF\TIP\{4FDCA82D-…}\LanguageProfile\0x00000804` 都在，**输入法列表里仍然可能没有青简**。

真正决定「列表里有没有这一条」的是当前用户的启用列表：

```
HKCU\Control Panel\International\User Profile\zh-Hans
```

里面每条是 `0804:{CLSID}{ProfileGUID}`，值 1 = 默认、2 = 备用、3 = 备用。
青简不在里面时，记事本这类应用**根本不会加载 `qingjian_tsf.dll`**——按键全归微软拼音，
表现就是「选中了输入法却打不出字」（其实选中的是别的输入法）。

### 怎么确认

```powershell
# 进程有没有加载 DLL
tasklist /m qingjian_tsf.dll /NH

# 列表里有什么（PowerShell 7 上 Get-WinUserLanguageList 可能因封送错误不可用，用 5.1）
cmd /c "powershell.exe -NoProfile -Command ""Get-WinUserLanguageList | % { $_.InputMethodTips }"""
```

正确做法是在**系统设置 → 时间和语言 → 语言和区域 → 中文(简体) → 语言选项 → 键盘 → 添加键盘**里加，
不要手改注册表那一条（改错了会让整个语言配置异常）。

## 4. Server 与 DLL 必须同一份源码编出来

`qingjian_core::Candidate` 是线上格式的一部分。Core 加一个枚举变体，老 DLL 就解不出整条帧、
把按键原样放行——症状同样是「突然只能打英文」，日志里只有一句 `unknown variant`。
**协议版本对不上时 Server 只警告、不拒绝**，别指望它兜住。

改了一个就两个一起换（`cargo build -p qingjian-windows-server -p qingjian-windows-tsf`），
换完把加载着旧 DLL 的应用关掉重开（`tasklist /m qingjian_tsf.dll` 看谁还挂着）。

## 5. `test-local.ps1` 的注册判据会误报

脚本注册完会比对 `InprocServer32` 与期望路径，不一致就打印

```
！注册可能没生效：… （regsvr32 退出码 1）
```

这台机器上出现过「脚本报退出码 1，手工执行同样的 `regsvr32 /s` 返回 0，且注册表确实已指向新 DLL」。
`regsvr32` 的退出码本来就不可信（脚本注释里也写了实测成功仍返回 3）。
**以注册表的值为准**，别以退出码为准；脚本自己也只警告不中断。

## 6. 别把「预存在的测试失败」当成自己改坏的

`cargo test -p qingjian-platform --lib` 在这台机器上有 **29 个失败**，全是
`code_schemes` / `aux_schemes` / `config` 里那些要写临时目录的用例：

```
panicked at crates\qingjian-platform\src\code_schemes\tests.rs:13:35:
called `Result::unwrap()` on an `Err` value: Os { code: 5, kind: PermissionDenied, message: "拒绝访问。" }
```

原因是这些测试用 `std::env::temp_dir()`（不受 `QINGJIAN_VARIANT` 影响的普通路径）建目录被拒。

**验证方法是比对基线**，不是猜：

```powershell
git stash push -u -m base
cargo test -p qingjian-platform --lib 2>&1 | Select-String 'test result'
git stash pop
```

基线是 `72 passed; 29 failed`。改动后失败数不变、通过数只增加自己新加的测试，就说明没引入新问题。

## 7. 两套产物抢同一套标识（已解决，别退回去）

正式版与开发版过去共用 CLSID / 管道 / 互斥体，同机只能活一个，症状同样是「切过去没反应」。
现在用 `QINGJIAN_VARIANT=dev` 编出标识全错开的一套，可以并存，见
[windows-dev-variant.md](windows-dev-variant.md)。

**注意 target 目录是共用的**：编过 dev 版之后再编正式版，记得把 `QINGJIAN_VARIANT` 去掉，
否则会继续出 dev 版。验证办法是查 DLL 里有没有 `qingjian-dev` 这个字符串。
