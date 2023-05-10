use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser};
use nix::sys::ptrace;
use nix::unistd::{fork, ForkResult, Pid};

use lurk_cli::args::{ArgCommand, Args};
use lurk_cli::{run_tracee, Tracer};

fn main() -> Result<()> {
    let config = Args::parse();
    let pid = if let Some(ArgCommand::Command(command)) = &config.command {
        if command.is_empty() {
            Args::command().print_help()?;
            return Ok(());
        }
        if config.attach.is_some() {
            bail!("The -p/--attach option cannot be used with a command");
        }
        // FIXME: I suspect this breaks Rust's safety: fork() spawn a thread and that thread
        //        is accessing the same memory as the parent thread (command/env/username/config)
        match unsafe { fork() } {
            Ok(ForkResult::Child) => return run_tracee(command, &config.env, &config.username),
            Ok(ForkResult::Parent { child }) => child,
            Err(err) => bail!("fork() failed: {err}"),
        }
    } else if let Some(pid) = config.attach {
        let pid = Pid::from_raw(pid);
        ptrace::attach(pid).with_context(|| format!("Unable to attach to process {pid}"))?;
        pid
    } else {
        Args::command().print_help()?;
        return Ok(());
    };

    Tracer::new(pid, config)?.run_tracer()
}
