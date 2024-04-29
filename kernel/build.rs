use build_config::{Config, LogLevel, MemoryMode};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let workspace_root = Path::new(env!("CARGO_RUSTC_CURRENT_DIR"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let cfg_path = workspace_root.join(env!("K23_CONFIG"));
    let cfg = Config::from_file(&cfg_path).expect("failed to parse build config");

    make_kconfig(&cfg, &out_dir)?;

    println!(
        "cargo:rustc-link-arg=-T{}",
        cfg.kernel.linker_script.display()
    );
    println!("cargo:rerun-if-changed={}", cfg_path.display());

    Ok(())
}

fn make_kconfig(cfg: &Config, out_dir: &Path) -> anyhow::Result<()> {
    let stack_size_pages = cfg.kernel.stack_size_pages;
    let trap_stack_size_pages = cfg.kernel.trap_stack_size_pages;
    let uart_baud_rate = cfg.kernel.uart_baud_rate;
    let log_level = match cfg.kernel.log_level {
        LogLevel::Error => "::log::Level::Error",
        LogLevel::Warn => "::log::Level::Warn",
        LogLevel::Info => "::log::Level::Info",
        LogLevel::Debug => "::log::Level::Debug",
        LogLevel::Trace => "::log::Level::Trace",
    };

    let memory_mode = match cfg.memory_mode {
        MemoryMode::Riscv64Sv39 => "::vmm::Riscv64Sv39",
        MemoryMode::Riscv64Sv48 => "::vmm::Riscv64Sv48",
        MemoryMode::Riscv64Sv57 => "::vmm::Riscv64Sv57",
    };

    let mut file = File::create(out_dir.join("kconfig.rs"))?;
    writeln!(
        file,
        r#"// Generated by build.rs, do not touch!
    pub const STACK_SIZE_PAGES: usize = {stack_size_pages};
    pub const TRAP_STACK_SIZE_PAGES: usize = {trap_stack_size_pages};
    pub const LOG_LEVEL: ::log::Level = {log_level};
    pub const UART_BAUD_RATE: u32 = {uart_baud_rate};
    #[allow(non_camel_case_types)]
    pub type MEMORY_MODE = {memory_mode};
    pub const PAGE_SIZE: usize = <MEMORY_MODE as ::vmm::Mode>::PAGE_SIZE;
    "#
    )?;

    Ok(())
}
