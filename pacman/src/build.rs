use crate::compile;
use crate::config;
use crate::config::Profile;
use crate::remote;
use crate::run;

pub fn run(path: &str, profile: &str, _args: Vec<String>, debug: bool) {
    let config = config::read(path);
    let profile = match config.profile.get(profile) {
        Some(prof) => (profile, prof),
        None => {
            println!("Profile \"{}\" not found", profile);
            std::process::exit(1);
        }
    };
    if let config::ProjectKind::Lib = profile.1.kind {
        println!("Cannot run a library");
        std::process::exit(1);
    }
    // build dependencies
    build_deps(&profile.1, profile.1._3rdparty as usize);
    // compile
    if compile::compile(path, profile) {
        run::run(path, &profile, &_args, debug);
    }
}

pub fn build(path: &str, profile: &str) -> bool {
    let config = config::read(path);
    let profile = match config.profile.get(profile) {
        Some(prof) => (profile, prof),
        None => {
            println!("Profile \"{}\" not found", profile);
            std::process::exit(1);
        }
    };
    // build dependencies
    build_deps(&profile.1, config._3rdparty as usize);
    // compile
    compile::compile(path, profile)
}

/// Build dependencies for a profile
pub fn build_deps(profile: &Profile, _3rdparty: usize) {
    for dep in &profile.dependencies {
        let mut path = remote::path(&dep.1.path, &"latest");
        // exists?
        if !std::path::Path::new(&path).exists() {
            if remote::is_remote(&dep.1.path) {
                path = remote::install(&dep.1.path, &"latest");
            } else {
                println!("Dependency {} not found", dep.1.path);
                std::process::exit(1);
            }
        }
        // is it a package?
        if config::contains(&path) {
            // read config
            let config = config::read(&path);
            // check 3rdparty level
            // levels: 0 - allow, 1 - std, 2 - sandboxed, 3 - deny
            let this_3rdparty = config._3rdparty as usize;
            // only for debug
            if this_3rdparty < _3rdparty {
                println!("Dependency {} is not allowed", dep.1.path);
                println!(
                    "{} is \"{}\" level, but current project allows \"{}\" level",
                    dep.1.path,
                    config::_3rdparty::to_str(this_3rdparty),
                    config::_3rdparty::to_str(_3rdparty)
                );
                std::process::exit(1);
            }
            // get profile
            // todo: get profile (default profile for now)
            let profile = match config.profile.get("default") {
                Some(prof) => (dep.0, prof),
                None => {
                    println!("Profile \"{}\" not found in {}", "default", path);
                    std::process::exit(1);
                }
            };
            // build dependencies
            build_deps(&profile.1, this_3rdparty);
            // compile
            compile::compile(&path, (profile.0, profile.1));
        } else {
            // err
            println!("Dependency {} is not a package", dep.1.path);
            std::process::exit(1);
        }
    }
}

/// Restore a project if cannot compile correctly
/// delete target directory
pub fn restore(path: &str, profile: &str, compile: bool, run: bool, args: Vec<String>, debug: bool) {
    let config = config::read(path);
    let profile = match config.profile.get(profile) {
        Some(prof) => (profile, prof),
        None => {
            println!("Profile \"{}\" not found", profile);
            std::process::exit(1);
        }
    };
    // delete target directory
    let profile_path = std::path::Path::new(path).join("target").join(profile.0);
    if profile_path.exists() {
        std::fs::remove_dir_all(&profile_path).unwrap();
    }
    // compile
    if compile || run {
        build_deps(&profile.1, profile.1._3rdparty as usize);
        if run && compile::compile(path, profile){
            run::run(path, &profile, &args, debug);
        }
    }
}


pub fn path_to_exe(path: &str, profile: &(&str, &Profile)) -> String {
    if let config::ProjectKind::Lib = profile.1.kind {
        println!("Executable not found. Cannot run a library");
        std::process::exit(1);
    }
    let path = std::path::Path::new(path).join("target").join(profile.0).join("out.rdbin");
    if !path.exists() {
        println!("Executable not found. Compile the project first.");
        std::process::exit(1);
    }
    let path = match path.to_str() {
        Some(path) => path,
        None => {
            println!("Failed to convert path to string.");
            return "".to_string();
        }
    };
    path.to_string()
}