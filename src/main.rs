use crate::ast_walker::AstWalker;
use crate::dir_walker::get_dir_walker;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tracing::{info, warn};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

mod ast_walker;
mod dir_walker;

#[derive(Clone, Debug, StructOpt)]
pub struct Config {
    #[structopt(long = "manifest-path")]
    manifest_path: Option<PathBuf>,
}

pub fn get_analysis(root: PathBuf) {
    info!("Analysing project in {}", root.display());
    for e in get_dir_walker(root.clone()) {
        analyse_package(e.path(), &root);
    }
}

/// Analyses a package of the target crate.
fn analyse_package(path: &Path, root: &Path) {
    if let Some(file) = path.to_str() {
        let skip_cause_test = path.starts_with(root.join("tests"));
        let skip_cause_example = path.starts_with(root.join("examples"));
        if !(skip_cause_test || skip_cause_example) {
            if let Ok(walker) = AstWalker::new(path.to_path_buf()) {
                walker.process();
            }
        }
    }
}

pub fn setup_logging() {
    let base_exceptions = |env: EnvFilter| {
        env.add_directive("doc_panic_checker=info".parse().unwrap())
            .add_directive(LevelFilter::INFO.into())
    };
    let filter = match std::env::var_os("RUST_LOG").map(|s| s.into_string()) {
        Some(Ok(env)) => {
            let mut filter = base_exceptions(EnvFilter::new(""));
            for s in env.split(',').into_iter() {
                match s.parse() {
                    Ok(d) => filter = filter.add_directive(d),
                    Err(err) => println!("WARN ignoring log directive: `{}`: {}", s, err),
                };
            }
            filter
        }
        _ => base_exceptions(EnvFilter::from_env("RUST_LOG")),
    };

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::ERROR)
        .with_env_filter(filter)
        .init();
}

fn main() {
    setup_logging();
    let config = Config::from_args();
    let root = config
        .manifest_path
        .map(|x| x.canonicalize().ok())
        .flatten()
        .map(|x| x.parent().map(|x| x.to_path_buf()).unwrap_or_default())
        .unwrap_or_default();

    get_analysis(root);
}
