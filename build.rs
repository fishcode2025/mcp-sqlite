fn main() {
    // 告诉Cargo如果README.md或docs目录下的文件发生变化，则重新构建
    println!("cargo:rerun-if-changed=README.md");
    println!("cargo:rerun-if-changed=docs/");

    // 如果是docs.rs环境，设置特殊标志
    if std::env::var("DOCS_RS").is_ok() {
        println!("cargo:rustc-cfg=docsrs");
    }
}
