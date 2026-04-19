fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DATABASE_URL");
    println!("cargo:rerun-if-env-changed=SQLX_OFFLINE");

    if std::env::var_os("DATABASE_URL").is_none() && std::env::var_os("SQLX_OFFLINE").is_none() {
        println!("cargo:rustc-env=SQLX_OFFLINE=true");
    }
}
