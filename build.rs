//! Сборка GNU-тулчейном Windows без полного MinGW.
//!
//! rustup поставляет не все import-библиотеки MinGW: например, нет
//! `libshlwapi.a`, которая нужна крейту `webbrowser` (зависимость eframe).
//! GNU ld умеет линковаться прямо с системными DLL, поэтому добавляем System32
//! в конец путей поиска — встроенные import-библиотеки остаются в приоритете.

fn main() {
    println!("cargo:rerun-if-env-changed=WINDIR");
    let target_is = |key: &str, value: &str| std::env::var(key).is_ok_and(|v| v == value);
    if target_is("CARGO_CFG_TARGET_OS", "windows")
        && target_is("CARGO_CFG_TARGET_ENV", "gnu")
        && let Some(windir) = std::env::var_os("WINDIR")
    {
        let system32 = std::path::Path::new(&windir).join("System32");
        println!("cargo:rustc-link-arg=-L{}", system32.display());
    }
}
