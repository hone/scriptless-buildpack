use failure::Error;
use libbuildpack::Detect;
use scriptless_buildpack::buildpack::Scriptless;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    platform: PathBuf,
    #[structopt(parse(from_os_str))]
    plan: PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Cli::from_args();

    let detect = Detect::new(args.platform, args.plan)?;
    let scriptless = Scriptless::load_toml()?;

    if let Some(detect_script) = scriptless.buildpack.detect {
        if !detect_script.run.is_empty() {
            println!("Running Detect Commands:");

            for script in detect_script.run {
                let args_string = format!("-c {}", script);
                let args = args_string.split_ascii_whitespace();
                let mut cmd = Command::new("sh")
                    .args(args)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()?;

                let status = cmd.wait()?;
                if !status.success() {
                    println!("Failed to run command");
                    std::process::exit(detect.fail());
                }
            }
        }
    }

    // TODO add support for contractual build plan
    detect.pass(None)?;

    Ok(())
}
