use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    if let Err(err) = run() {
        eprintln!("[error] {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cmd = env::args().nth(1).unwrap_or_else(|| "all".to_string());
    let root = workspace_root()?;

    match cmd.as_str() {
        "all" => {
            exec(&root, "cargo", &["fmt", "--all", "--", "--check"])?;
            exec(&root, "cargo", &["check", "--workspace"])?;
            exec(&root, "cargo", &["test", "--workspace"])?;
            exec(&root, "./scripts/check-query-compile.sh", &[])?;
            exec(&root, "./scripts/check-openapi-schema.sh", &[])?;
            exec(&root, "./scripts/validate_database_migrations.sh", &[])?;
            exec(&root, "./scripts/seed-demo.sh", &[])?;
            println!("[ok] xtask all completed");
            Ok(())
        }
        "fmt" => exec(&root, "cargo", &["fmt", "--all", "--", "--check"]),
        "lint" => exec(&root, "cargo", &["check", "--workspace"]),
        "test" => exec(&root, "cargo", &["test", "--workspace"]),
        "query-compile-check" => exec(&root, "./scripts/check-query-compile.sh", &[]),
        "openapi-check" => exec(&root, "./scripts/check-openapi-schema.sh", &[]),
        "migrate-check" => exec(&root, "./scripts/validate_database_migrations.sh", &[]),
        "seed" => exec(&root, "./scripts/seed-demo.sh", &[]),
        other => Err(format!(
            "unknown xtask command: {other}. expected one of: all, fmt, lint, test, query-compile-check, openapi-check, migrate-check, seed"
        )),
    }
}

fn workspace_root() -> Result<PathBuf, String> {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(|e| format!("env error: {e}"))?);
    manifest_dir
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| "cannot determine workspace root".to_string())
}

fn exec(root: &PathBuf, program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|e| format!("failed to execute `{program}`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "`{program} {}` failed with status {status}",
            args.join(" ")
        ))
    }
}
