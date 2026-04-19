use std::env;
use std::path::PathBuf;
use std::process::Command;

const DEFAULT_DATABASE_URL: &str = "postgres://datab:datab_local_pass@127.0.0.1:5432/datab";

fn main() {
    if let Err(err) = run() {
        eprintln!("[error] {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let cmd = args.first().cloned().unwrap_or_else(|| "all".to_string());
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
        "sqlx" => run_sqlx(&root, &args[1..]),
        other => Err(format!(
            "unknown xtask command: {other}. expected one of: all, fmt, lint, test, query-compile-check, openapi-check, migrate-check, seed, sqlx"
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

fn run_sqlx(root: &PathBuf, args: &[String]) -> Result<(), String> {
    let normalized_args = normalize_sqlx_args(args);
    let mut cmd = Command::new("cargo-sqlx");
    cmd.arg("sqlx").args(&normalized_args).current_dir(root);

    if env::var_os("DATABASE_URL").is_none() {
        cmd.env("DATABASE_URL", DEFAULT_DATABASE_URL);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("failed to execute `cargo-sqlx`: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "`cargo-sqlx sqlx {}` failed with status {status}",
            normalized_args.join(" ")
        ))
    }
}

fn normalize_sqlx_args(args: &[String]) -> Vec<String> {
    let mut normalized = args.to_vec();
    let is_prepare = normalized.first().is_some_and(|arg| arg == "prepare");

    if is_prepare && !has_sqlx_scope_arg(&normalized[1..]) {
        normalized.push("--workspace".to_string());
    }

    normalized
}

fn has_sqlx_scope_arg(args: &[String]) -> bool {
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        if arg == "--workspace" || arg == "--package" {
            return true;
        }
        if arg == "-p" {
            return true;
        }
        if arg.starts_with("-p") && arg.len() > 2 {
            return true;
        }
        if arg == "--" {
            break;
        }

        if arg == "--package" || arg == "-p" {
            let _ = iter.next();
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{has_sqlx_scope_arg, normalize_sqlx_args};

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn prepare_adds_workspace_scope_by_default() {
        assert_eq!(
            normalize_sqlx_args(&strings(&["prepare"])),
            strings(&["prepare", "--workspace"])
        );
    }

    #[test]
    fn prepare_keeps_explicit_workspace_scope() {
        assert_eq!(
            normalize_sqlx_args(&strings(&["prepare", "--workspace"])),
            strings(&["prepare", "--workspace"])
        );
    }

    #[test]
    fn prepare_keeps_explicit_package_scope() {
        assert_eq!(
            normalize_sqlx_args(&strings(&["prepare", "-p", "db"])),
            strings(&["prepare", "-p", "db"])
        );
        assert!(has_sqlx_scope_arg(&strings(&["-p", "db"])));
        assert!(has_sqlx_scope_arg(&strings(&["--package", "db"])));
    }
}
