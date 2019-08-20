use failure::Error;
use libbuildpack::Build;
use scriptless_buildpack::buildpack::Scriptless;
use std::{fs, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    layers: PathBuf,
    #[structopt(parse(from_os_str))]
    platform: PathBuf,
    #[structopt(parse(from_os_str))]
    plan: PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Cli::from_args();
    let args_array = [&args.layers, &args.platform, &args.plan];

    let mut build = Build::new(&args.layers, &args.platform, &args.plan)?;
    // scratch directory
    let tmpdir = tempdir::TempDir::new("scriptless")?;

    let scriptless = Scriptless::load_toml()?;
    if let Some(build_script) = scriptless.buildpack.build {
        if !build_script.run.is_empty() {
            println!("Running build.run Script");

            let mut cmd = build_script.run.execute(&args_array)?;

            let status = cmd.wait()?;
            if !status.success() {
                println!("Failed to run command");
                std::process::exit(build.fail(status.code().unwrap()));
            }
        }

        for layer_toml in build_script.layers {
            let layer_tmpdir = tmpdir.path().join("layers");
            fs::create_dir_all(&layer_tmpdir)?;

            let mut layer = build.layers.add(&layer_toml.id)?;
            layer.read_metadata()?;

            if !layer_toml.run.is_empty()
                && !layer_toml.should_rebuild(layer.config.cache, &layer.config.metadata)
            {
                println!("Running Layer Script: {}", layer_toml.id);

                let mut cmd = layer_toml.run.execute(&args_array)?;

                let status = cmd.wait()?;
                if !status.success() {
                    println!("Failed to run layers command");
                    std::process::exit(build.fail(status.code().unwrap()));
                }
            }

            layer.config(|c| {
                c.cache = layer_toml.cache;
                c.launch = layer_toml.launch;
                c.build = layer_toml.build;
            })?;

            if !layer_toml.metadata.is_empty() {
                // partially consumes scriptless layer toml
                for (key, value) in layer_toml.metadata.into_iter() {
                    layer.config.metadata.insert(key, value);
                }
                layer.write_metadata()?;
            }

            if !layer_toml.env.is_empty() {
                for (key, value) in &layer_toml.env {
                    layer.envs.shared.append.set_var(key, value);
                }
                layer.write_envs()?;
            }

            for profile in &layer_toml.profile {
                layer.write_profile_d(&profile.name, &profile.script)?;
            }

            for process in &build_script.launch.processes {
                build
                    .layers
                    .launch
                    .add_process(&process.r#type, &process.command);
            }
        }
    }

    Ok(())
}
