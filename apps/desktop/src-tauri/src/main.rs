// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  // Install a panic hook that logs the panic details to stderr + ~/bcip-panic.log
  // before the default abort. This makes release crashes diagnosable.
  std::panic::set_hook(Box::new(|info| {
    let payload = info
      .payload()
      .downcast_ref::<&str>()
      .map(|s| s.to_string())
      .or_else(|| info.payload().downcast_ref::<String>().cloned())
      .unwrap_or_else(|| "unknown panic".to_string());

    let location = info
      .location()
      .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
      .unwrap_or_else(|| "unknown location".to_string());

    let msg = format!(
      "[bcip-agent PANIC] {}\n  at {}\n  backtrace:\n{}",
      payload,
      location,
      std::backtrace::Backtrace::capture()
    );

    eprintln!("{msg}");

    // Also write to a file so we can read it even if stderr is not captured
    if let Some(home) = dirs::home_dir() {
      let _ = std::fs::write(home.join("bcip-panic.log"), &msg);
    }
  }));

  app_lib::run();
}
