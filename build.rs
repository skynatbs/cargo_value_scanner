use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if let Ok(output) = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
    {
        if output.status.success() {
            if let Ok(tag) = String::from_utf8(output.stdout) {
                let tag = tag.trim();
                if !tag.is_empty() {
                    println!("cargo:rustc-env=GIT_TAG={tag}");
                }
            }
        }
    }
}
