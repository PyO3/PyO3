use crate::cli;
use crate::cli::DocOpts;
use std::process::Command;
//--cfg docsrs --Z unstable-options --document-hidden-items

pub fn run(opts: DocOpts) -> anyhow::Result<()> {
    let mut flags = Vec::new();

    if !opts.stable {
        flags.push("--cfg docsrs");
    }
    if opts.internal {
        flags.push("--Z unstable-options");
        flags.push("--document-hidden-items");
    }
    flags.push("-Dwarnings");

    std::env::set_var("RUSTDOCFLAGS", flags.join(" "));
    cli::run(
        Command::new("cargo")
            .args(if opts.stable { None } else { Some("+nightly") })
            .arg("doc")
            .arg("--lib")
            .arg("--no-default-features")
            .arg("--features=full")
            .arg("--no-deps")
            .arg("--workspace")
            .args(if opts.internal {
                ["--document-private-items"].as_slice()
            } else {
                ["--exclude=pyo3-macros", "--exclude=pyo3-macros-backend"].as_slice()
            })
            .args(if opts.stable {
                &[][..]
            } else {
                &[
                    "-Z",
                    "unstable-options",
                    "-Z",
                    "rustdoc-scrape-examples=examples",
                ]
            })
            .args(if opts.open { Some("--open") } else { None }),
    )?;

    Ok(())
}

// " --lib  --features=full --no-deps --workspace --open --exclude pyo3-macros --exclude pyo3-macros-backend"
/*
pyo3_doc =      "
pyo3_doc_scrape = "doc --lib --no-default-features --features=full --no-deps --workspace --open --exclude pyo3-macros --exclude pyo3-macros-backend
 "
*/
