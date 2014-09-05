#![feature(phase)]

#[phase(plugin)]
extern crate docopt_macros;

extern crate cargo;
extern crate docopt;
extern crate glob;
extern crate serialize;

use cargo::core::{MultiShell, Package, Source};
use cargo::sources::{PathSource};
use cargo::util::{CliResult, CliError};
use cargo::util::important_paths::{find_root_manifest_for_cwd};
use docopt::FlagParser;
use serialize::{Encodable, json};

docopt!(Args, "
Usage: scan --repo REPO --subdir SUBDIR

Options:
    --repo REPO             URI of the repository.
    --subdir SUBDIR         Path of the subdirectory.
")

#[allow(dead_code, non_snake_case)]
#[deriving(Encodable)]
struct SourceUnit {
    Name: String,
    Type: String,
    Repo: Option<String>,
    Globs: Vec<String>,
    Files: Vec<String>,
    Dir: String,
    Dependencies: Vec<String>
}

fn build_source_unit() -> SourceUnit {
    unreachable!()
}

fn construct_source_unit() -> Result<[SourceUnit, ..1], cargo::util::errors::CliError> {
    let root = try!(find_root_manifest_for_cwd(None))
        .dir_path();
    let mut source = try!(PathSource::for_path(&root)
        .map_err(|e| CliError::new(e.description(), 1)));
    try!(source.update().map_err(|err| CliError::new(err.description(), 1)));
    let package = try!(source.get_root_package()
        .map_err(|err| CliError::from_boxed(err, 1)));
    let manifest = package.get_manifest();
    let dependencies = manifest.get_dependencies();
    
    let glob = root.join("src").join("**.rs")
        .path_relative_from(&std::os::getcwd()).unwrap();

    Ok([SourceUnit {
        Name: manifest.get_name().to_string(),
        Type: "RustCargoPackage".to_string(),
        Repo: None,
        Globs: vec![glob.as_str().unwrap().to_string()],
        Files: glob::glob(glob.as_str().unwrap())
            .map(|path| {
                path.path_relative_from(&std::os::getcwd())
                    .unwrap().as_str().unwrap().to_string()
            })
            .collect(),
        Dir: root.path_relative_from(&std::os::getcwd())
            .unwrap().as_str().unwrap().to_string(),
        Dependencies: dependencies.iter()
            .map(|dependency| dependency.get_source_id().to_url())
            .collect()
    }])
}

fn main() {
    let args: Args = FlagParser::parse().unwrap_or_else(|e| e.exit());
    let source_units = construct_source_unit().unwrap();
    let mut stdout = std::io::stdio::stdout();
    let mut encoder = json::PrettyEncoder::new(&mut stdout);
    source_units.encode(&mut encoder);
}
