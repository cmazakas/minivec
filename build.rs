fn main() {
  let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());

  let is_nightly = std::process::Command::new(rustc)
    .arg("--version")
    .output()
    .ok()
    .and_then(|output| String::from_utf8(output.stdout).ok())
    .map_or(false, |output| output.contains("nightly"));

  if is_nightly {
    println!("cargo:rustc-cfg=allocator_api");
  }
}
