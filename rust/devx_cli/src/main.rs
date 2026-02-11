use clap::{ColorChoice, CommandFactory, Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, ExitCode};

#[derive(Parser, Debug)]
#[command(
    name = "devx_cli",
    version = "0.2.0",
    about = "Tiny DevEx demo tool (automation + CI patterns)",
    arg_required_else_help = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Show help (explicit command for README-friendly UX)
    Help,

    /// Print an environment report (rust/python/pytest)
    Doctor,

    /// Run repo automation tasks
    Run {
        #[command(subcommand)]
        task: RunTask,
    },

    /// Print tool help text (used by docs generator)
    #[command(hide = true)]
    EmitHelp,
}

#[derive(Subcommand, Debug, Clone)]
enum RunTask {
    /// Generate docs (COMMANDS.md) deterministically
    Docs,
    /// Run Python tests (pytest)
    Test,
    /// Run full CI: fmt + clippy + tests + docs sync
    Ci,
    /// Format Rust code
    Fmt,
    /// Lint Rust code (clippy -D warnings)
    Lint,
    /// Build Rust CLI
    Build,
}

fn main() -> ExitCode {
    match Cli::parse().cmd {
        Cmd::Help => {
            print_long_help();
            ExitCode::SUCCESS
        }
        Cmd::EmitHelp => {
            // Stable help output for docs generator
            print_long_help();
            ExitCode::SUCCESS
        }
        Cmd::Doctor => doctor(),
        Cmd::Run { task } => match task {
            RunTask::Docs => run_docs(),
            RunTask::Test => run_tests(),
            RunTask::Ci => run_ci(),
            RunTask::Fmt => run_fmt(),
            RunTask::Lint => run_lint(),
            RunTask::Build => run_build(),
        },
    }
}

fn print_long_help() {
    // Make help output deterministic across environments (CI vs local)
    let mut cmd = Cli::command();
    cmd = cmd.color(ColorChoice::Never).term_width(100);
    print!("{}", cmd.render_long_help());
}

fn doctor() -> ExitCode {
    let mut ok = true;

    ok &= check_cmd("rustc", &["--version"]);
    ok &= check_cmd("cargo", &["--version"]);

    ok &= check_any_cmd(&["python", "python3"], &["--version"]);

    // Prefer python -m pytest for consistency across installs
    ok &= check_any_cmd(&["python", "python3"], &["-m", "pytest", "--version"]);

    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn run_docs() -> ExitCode {
    let root = match find_repo_root() {
        Some(r) => r,
        None => {
            eprintln!("Could not find repo root (.git). Run inside the repo.");
            return ExitCode::from(2);
        }
    };

    // Ensure binary exists for the generator
    let rust_dir = root.join("rust/devx_cli");
    if !run_ok(Command::new("cargo").arg("build").current_dir(&rust_dir)) {
        return ExitCode::from(2);
    }

    let python = pick_cmd(&["python", "python3"]).unwrap_or_else(|| "python3".to_string());
    let script = root.join("python/tools/tools/gen_docs.py");

    if run_ok(Command::new(python).arg(script).current_dir(&root)) {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn run_tests() -> ExitCode {
    let root = match find_repo_root() {
        Some(r) => r,
        None => {
            eprintln!("Could not find repo root (.git). Run inside the repo.");
            return ExitCode::from(2);
        }
    };

    let rust_dir = root.join("rust/devx_cli");
    let python_dir = root.join("python/tools");
    let python = pick_cmd(&["python", "python3"]).unwrap_or_else(|| "python3".to_string());

    let mut ok = true;

    ok &= run_ok(Command::new("cargo").arg("test").current_dir(&rust_dir));
    ok &= run_ok(
        Command::new(python)
            .args(["-m", "pytest", "-q"])
            .current_dir(&python_dir),
    );

    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn run_ci() -> ExitCode {
    let root = match find_repo_root() {
        Some(r) => r,
        None => {
            eprintln!("Could not find repo root (.git). Run inside the repo.");
            return ExitCode::from(2);
        }
    };

    let rust_dir = root.join("rust/devx_cli");
    let python_dir = root.join("python/tools");
    let python = pick_cmd(&["python", "python3"]).unwrap_or_else(|| "python3".to_string());

    let mut ok = true;

    ok &= run_ok(
        Command::new("cargo")
            .args(["fmt", "--all", "--", "--check"])
            .current_dir(&rust_dir),
    );
    ok &= run_ok(
        Command::new("cargo")
            .args(["clippy", "--all-targets", "--", "-D", "warnings"])
            .current_dir(&rust_dir),
    );
    ok &= run_ok(Command::new("cargo").arg("test").current_dir(&rust_dir));

    // docs generation should be deterministic; tests enforce sync
    ok &= run_ok(
        Command::new(python.clone())
            .arg("python/tools/tools/gen_docs.py")
            .current_dir(&root),
    );
    ok &= run_ok(
        Command::new(python)
            .args(["-m", "pytest", "-q"])
            .current_dir(&python_dir),
    );

    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn run_fmt() -> ExitCode {
    let root = match find_repo_root() {
        Some(r) => r,
        None => {
            eprintln!("Could not find repo root (.git). Run inside the repo.");
            return ExitCode::from(2);
        }
    };

    let rust_dir = root.join("rust/devx_cli");

    if run_ok(Command::new("cargo").args(["fmt", "--all"]).current_dir(&rust_dir)) {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn run_lint() -> ExitCode {
    let root = match find_repo_root() {
        Some(r) => r,
        None => {
            eprintln!("Could not find repo root (.git). Run inside the repo.");
            return ExitCode::from(2);
        }
    };

    let rust_dir = root.join("rust/devx_cli");

    if run_ok(
        Command::new("cargo")
            .args(["clippy", "--all-targets", "--", "-D", "warnings"])
            .current_dir(&rust_dir),
    ) {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn run_build() -> ExitCode {
    let root = match find_repo_root() {
        Some(r) => r,
        None => {
            eprintln!("Could not find repo root (.git). Run inside the repo.");
            return ExitCode::from(2);
        }
    };

    let rust_dir = root.join("rust/devx_cli");

    if run_ok(Command::new("cargo").arg("build").current_dir(&rust_dir)) {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn run_ok(cmd: &mut Command) -> bool {
    match cmd.status() {
        Ok(s) => s.success(),
        Err(e) => {
            eprintln!("Failed to run command: {e}");
            false
        }
    }
}

fn check_cmd(cmd: &str, args: &[&str]) -> bool {
    match Command::new(cmd).args(args).status() {
        Ok(s) if s.success() => true,
        _ => {
            eprintln!("Missing or failing: {cmd}");
            false
        }
    }
}

fn check_any_cmd(cmds: &[&str], args: &[&str]) -> bool {
    for c in cmds {
        if matches!(Command::new(c).args(args).status(), Ok(s) if s.success()) {
            return true;
        }
    }
    eprintln!("Missing or failing one of: {:?}", cmds);
    false
}

fn pick_cmd(cmds: &[&str]) -> Option<String> {
    for c in cmds {
        if matches!(Command::new(c).arg("--version").status(), Ok(s) if s.success()) {
            return Some((*c).to_string());
        }
    }
    None
}

fn find_repo_root() -> Option<PathBuf> {
    let mut cur = std::env::current_dir().ok()?;
    loop {
        if cur.join(".git").exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}
