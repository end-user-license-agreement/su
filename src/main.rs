// `CommandExt::groups` isn't stablized yet.
#![feature(setgroups)]

use {
    clap::Parser,
    std::{ffi::OsString, os::unix::process::CommandExt, process::Command},
};

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, short)]
    user: Option<String>,

    #[arg(default_value = "/bin/sh", trailing_var_arg = true)]
    command: Vec<OsString>,
}

fn main() {
    let args = Args::parse();
    let user = args
        .user
        .as_deref()
        .and_then(users::get_user_by_name)
        .unwrap_or_else(|| {
            // hopefully never happens on a normal system.
            users::get_user_by_uid(0)
                .unwrap_or_else(|| panic!("no root account found on this system"))
        });

    let groups = user
        .groups()
        .into_iter()
        .flatten()
        .map(|group| group.gid())
        .collect::<Vec<_>>();

    let mut args = args.command.into_iter();

    // SAFETY: #[arg(default_value = "/bin/sh")] ensures this always exists.
    let program = unsafe { args.next().unwrap_unchecked() };

    let mut command = Command::new(program);

    command
        .args(args)
        .uid(user.uid())
        .gid(user.primary_group_id())
        .groups(&groups);

    // NOTE: `exec` returns `io::Error` on failure, else it doesn't return.
    let error = command.exec();

    eprintln!("{}: {error}", env!("CARGO_PKG_NAME"));
}
