use std::process::Command;

fn main() {
    // Re-run if any Rust source or the input CSS changes
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=input.css");

    let status = Command::new("bunx")
        .args(["@tailwindcss/cli", "-i", "./input.css", "-o", "./assets/tailwind.css"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("warning: tailwindcss exited with {s}. Using existing assets/tailwind.css if available.");
        }
        Err(e) => {
            eprintln!("warning: could not run bunx tailwindcss: {e}. Using existing assets/tailwind.css if available.");
        }
    }
}
