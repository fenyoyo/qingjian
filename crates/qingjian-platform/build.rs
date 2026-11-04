//! 构建期变体：`QINGJIAN_VARIANT` 环境变量决定这一份产物是「正式版」还是「开发版」。
//!
//! 两套产物要能在同一台机器上共存（方便调试：正式版日常用、开发版随时换代码），
//! 就必须把系统里那些**全局唯一**的标识各分一套：COM 的 CLSID、TSF 语言 profile、
//! 命名管道、单实例互斥体、拉起互斥体、输入法显示名，以及用户数据目录与日志目录。
//! 只要有一处共用，两个 Server 就会抢同一个管道、两个 DLL 抢同一个 CLSID，
//! 表现为「切过去没反应」或「时好时坏」——那种失败是静默的，很难查。
//!
//! 缺省（不设该变量）是正式版，标识与历史版本完全一致，升级不受影响。
//! 开发版用 `QINGJIAN_VARIANT=dev` 构建，标识与显示名另有一套。
//!
//! 结论以 `cfg(qingjian_variant_dev)` 传进编译单元；具体取值集中在 `src/identity.rs`。
//! 用 cfg 而不是 `env!` 是因为常量里不能比较字符串（`str` 的 `PartialEq` 还不是 const trait）。

fn main() {
    println!("cargo:rerun-if-env-changed=QINGJIAN_VARIANT");
    // 自定义 cfg 要声明，否则 unexpected_cfgs 会告警，CI 的 -D warnings 会拦下来。
    println!("cargo::rustc-check-cfg=cfg(qingjian_variant_dev)");

    let variant = std::env::var("QINGJIAN_VARIANT").unwrap_or_default();
    let variant = variant.trim().to_ascii_lowercase();
    match variant.as_str() {
        // 正式版：不设，或者显式写 release / prod。
        "" | "release" | "prod" => {}
        "dev" => println!("cargo:rustc-cfg=qingjian_variant_dev"),
        // 拼错当成正式版太危险（两套产物会撞标识，症状还很隐蔽），直接让构建失败。
        other => panic!("QINGJIAN_VARIANT 只支持 dev（开发版）或不设（正式版），收到 {other:?}"),
    }
}
