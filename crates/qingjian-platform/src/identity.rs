//! 这一份产物在系统里的身份：正式版还是开发版，以及随之决定的那一组全局唯一标识。
//!
//! 变体由构建期环境变量 `QINGJIAN_VARIANT` 定（`dev` 为开发版，不设为正式版），见 `build.rs`。
//! 正式版的取值与历史版本逐字一致，所以升级不受影响；开发版每个标识都错开一格，
//! 两套产物才能同时注册、同时运行，各自有自己的 Server、管道、数据目录与输入法条目。
//!
//! **改这里的任何常量都等于换掉一个已发布产品的身份**：正式版那半边不能动，
//! 动了就是「升级后输入法从列表里消失」或「装两遍变成两条」。开发版那半边可以随便改。
//!
//! 判别变体的 `cfg(qingjian_variant_dev)` 由 `qingjian-platform` 的 build.rs 发出，
//! **只在本 crate 内有效**（rustc 的 cfg 不跨 crate 传播）。所以别处一律用这里的常量，
//! 不要在自己的 crate 里写 `cfg(qingjian_variant_dev)`。

/// 开发版构建（`QINGJIAN_VARIANT=dev`）。
pub const IS_DEV: bool = cfg!(qingjian_variant_dev);

/// 用户数据目录与日志目录用的文件夹名：`%APPDATA%\<它>`。
///
/// 两套分开，两个 Server 同时跑才不会争着写 `user.tsv` / `user-ngram.tsv`（互相覆盖学习数据）。
#[cfg(qingjian_variant_dev)]
pub const DIR_NAME: &str = "Qingjian-dev";
/// 用户数据目录与日志目录用的文件夹名：`%APPDATA%\<它>`。
#[cfg(not(qingjian_variant_dev))]
pub const DIR_NAME: &str = "Qingjian";

/// 输入法在系统里显示的名字（任务栏、Win+Space 列表）。
#[cfg(qingjian_variant_dev)]
pub const DISPLAY_NAME: &str = "青简 Dev";
/// 输入法在系统里显示的名字（任务栏、Win+Space 列表）。
#[cfg(not(qingjian_variant_dev))]
pub const DISPLAY_NAME: &str = "青简";

/// 文本服务的 CLSID（u128 形式，给 `GUID::from_u128` 用）。
#[cfg(qingjian_variant_dev)]
pub const CLSID: u128 = 0x7c3e5f1a_2b4d_4e8f_9a6c_1d2e3f4a5b6c;
/// 文本服务的 CLSID（u128 形式，给 `GUID::from_u128` 用）。
#[cfg(not(qingjian_variant_dev))]
pub const CLSID: u128 = 0x4fdca82d_e923_49bf_9e75_bb906b93b8bb;

/// [`CLSID`] 的注册表字符串形式，两者必须同步改。
#[cfg(qingjian_variant_dev)]
pub const CLSID_STR: &str = "{7C3E5F1A-2B4D-4E8F-9A6C-1D2E3F4A5B6C}";
/// [`CLSID`] 的注册表字符串形式，两者必须同步改。
#[cfg(not(qingjian_variant_dev))]
pub const CLSID_STR: &str = "{4FDCA82D-E923-49BF-9E75-BB906B93B8BB}";

/// TSF 语言 profile 的 GUID（u128），与 CLSID 一起决定「输入法列表里是哪一条」。
#[cfg(qingjian_variant_dev)]
pub const PROFILE: u128 = 0x9d4f2a1b_6c8e_4b3a_8f5d_2e7c1a9b4f6d;
/// TSF 语言 profile 的 GUID（u128），与 CLSID 一起决定「输入法列表里是哪一条」。
#[cfg(not(qingjian_variant_dev))]
pub const PROFILE: u128 = 0x8119f8e0_cf81_423b_9189_c0d7374324b3;

/// Server 与 DLL 之间的命名管道。两套各有各的，各自的 Server 才能同时监听。
#[cfg(qingjian_variant_dev)]
pub const PIPE_NAME: &str = r"\\.\pipe\qingjian-dev";
/// Server 与 DLL 之间的命名管道。两套各有各的，各自的 Server 才能同时监听。
#[cfg(not(qingjian_variant_dev))]
pub const PIPE_NAME: &str = r"\\.\pipe\qingjian";

/// Server 的单实例互斥体：占着就说明本变体已经有一个 Server 在跑。
#[cfg(qingjian_variant_dev)]
pub const SERVER_MUTEX: &str = r"Local\QingjianServer-dev";
/// Server 的单实例互斥体：占着就说明本变体已经有一个 Server 在跑。
#[cfg(not(qingjian_variant_dev))]
pub const SERVER_MUTEX: &str = r"Local\QingjianServer";

/// DLL 拉起 Server 用的跨进程互斥体（多个应用的 DLL 同时发现 Server 不在时只起一个）。
#[cfg(qingjian_variant_dev)]
pub const LAUNCH_MUTEX: &str = r"Local\QingjianServerLaunch-dev";
/// DLL 拉起 Server 用的跨进程互斥体（多个应用的 DLL 同时发现 Server 不在时只起一个）。
#[cfg(not(qingjian_variant_dev))]
pub const LAUNCH_MUTEX: &str = r"Local\QingjianServerLaunch";

/// 输入法图标文件的名字（写在 `%ProgramData%\Qingjian\` 下）。两套分开，互不覆盖。
#[cfg(qingjian_variant_dev)]
pub const ICON_FILE: &str = "qingjian-dev.ico";
/// 输入法图标文件的名字（写在 `%ProgramData%\Qingjian\` 下）。两套分开，互不覆盖。
#[cfg(not(qingjian_variant_dev))]
pub const ICON_FILE: &str = "qingjian.ico";

/// 设置程序窗口标题的后缀。正式版为空（标题就是各语言文案里的名字），开发版加「 Dev」，
/// 两个窗口同时开着时一眼分得清。跟着界面语言走，所以不用 [`DISPLAY_NAME`]（那个只有中文名）。
#[cfg(qingjian_variant_dev)]
pub const WINDOW_TITLE_SUFFIX: &str = " Dev";
/// 设置程序窗口标题的后缀，正式版没有。
#[cfg(not(qingjian_variant_dev))]
pub const WINDOW_TITLE_SUFFIX: &str = "";

/// 设置程序的单实例互斥体。按变体分开，两套设置程序才能同时开、各自编辑自己的配置。
///
/// 注意它与旧设置程序（`qingjian-settings-legacy.exe`）的互斥体本来就不同，见 `system::single_instance`。
#[cfg(qingjian_variant_dev)]
pub const SETTINGS_MUTEX: &str = r"Local\QingjianSettings-dev";
/// 设置程序的单实例互斥体。
#[cfg(not(qingjian_variant_dev))]
pub const SETTINGS_MUTEX: &str = r"Local\QingjianSettings";

#[cfg(test)]
mod tests {
    use super::{
        CLSID, CLSID_STR, DIR_NAME, DISPLAY_NAME, IS_DEV, PIPE_NAME, PROFILE, SERVER_MUTEX,
        SETTINGS_MUTEX, WINDOW_TITLE_SUFFIX,
    };

    /// 正式版的标识是已发布产品的身份，一个字节都不能变——变了就是升级后输入法消失。
    #[test]
    fn release_identity_is_frozen() {
        if IS_DEV {
            return;
        }
        assert_eq!(CLSID_STR, "{4FDCA82D-E923-49BF-9E75-BB906B93B8BB}");
        assert_eq!(CLSID, 0x4fdca82d_e923_49bf_9e75_bb906b93b8bb);
        assert_eq!(PROFILE, 0x8119f8e0_cf81_423b_9189_c0d7374324b3);
        assert_eq!(DIR_NAME, "Qingjian");
        assert_eq!(DISPLAY_NAME, "青简");
        assert_eq!(PIPE_NAME, r"\\.\pipe\qingjian");
        assert_eq!(SERVER_MUTEX, r"Local\QingjianServer");
        // 正式版设置程序的窗口标题与互斥体也不能变：前者是用户看到的，后者决定能不能开第二个
        assert_eq!(WINDOW_TITLE_SUFFIX, "");
        assert_eq!(SETTINGS_MUTEX, r"Local\QingjianSettings");
    }

    /// 开发版的每个标识都要与正式版错开，否则同机共存时会互相抢。
    #[test]
    fn dev_identity_differs_from_release() {
        if !IS_DEV {
            return;
        }
        assert_ne!(CLSID, 0x4fdca82d_e923_49bf_9e75_bb906b93b8bb);
        assert_ne!(PROFILE, 0x8119f8e0_cf81_423b_9189_c0d7374324b3);
        assert_ne!(PIPE_NAME, r"\\.\pipe\qingjian");
        assert_ne!(SERVER_MUTEX, r"Local\QingjianServer");
        assert_ne!(DIR_NAME, "Qingjian");
        assert_ne!(DISPLAY_NAME, "青简");
        // 设置程序：标题带后缀才分得清两个窗口，互斥体不同才能同时开
        assert_eq!(WINDOW_TITLE_SUFFIX, " Dev");
        assert_ne!(SETTINGS_MUTEX, r"Local\QingjianSettings");
    }
}
