#![allow(dead_code)]
#![deny(unused_must_use)]

use std::{env, path::PathBuf};

use xshell::cmd;

fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    match &args[..] {
        ["test", "all"] => test_all(),
        ["test", "host"] => test_host(),
        ["test", "host-target"] => test_host_target(),
        ["test", "target"] => test_target(),
        _ => {
            println!("USAGE cargo xtask test [all|host|host-target|target]");
            Ok(())
        }
    }
}

fn test_all() -> Result<(), anyhow::Error> {
    test_host()?;
    test_target()?;
    test_host_target()
}

fn test_host() -> Result<(), anyhow::Error> {
    let _p = xshell::pushd(root_dir())?;
    cmd!("cargo test --workspace --exclude host-target-tests").run()?;
    Ok(())
}

fn test_host_target() -> Result<(), anyhow::Error> {
    flash()?;

    let _p = xshell::pushd(root_dir())?;
    cmd!("cargo test -p host-target-tests").run()?;

    Ok(())
}

fn test_target() -> Result<(), anyhow::Error> {
    let _p = xshell::pushd(root_dir().join("cross"))?;
    cmd!("cargo test -p self-tests").run()?;
    Ok(())
}

fn flash() -> Result<(), anyhow::Error> {
    let _p = xshell::pushd(root_dir().join("cross"))?;
    cmd!("cargo flash --chip nRF52840_xxAA --release").run()?;
    Ok(())
}

fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();
    xtask_dir
}
