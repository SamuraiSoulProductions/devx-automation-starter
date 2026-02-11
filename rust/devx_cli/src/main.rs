use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sub = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match sub {
        "help" | "--help" | "-h" => help(),
        "version" => println!("devx_cli 0.1.0"),
        "emit-help" => emit_help(),
        _ => {
            eprintln!("Unknown command: {sub}\n");
            help();
            std::process::exit(2);
        }
    }
}

fn help() {
    println!(
r#"devx_cli — tiny DevEx demo tool

USAGE:
  devx_cli help
  devx_cli version
  devx_cli emit-help

WHY:
  Demonstrates repo automation patterns: docs generation + CI enforcement.
"#
    );
}

fn emit_help() {
    let out = Command::new(std::env::current_exe().unwrap())
        .arg("help")
        .output()
        .expect("failed to run self help");
    print!("{}", String::from_utf8_lossy(&out.stdout));
}
