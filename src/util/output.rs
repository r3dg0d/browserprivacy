use serde::Serialize;
use std::io::{self, Write};

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct OutputOpts {
    pub json: bool,
    pub quiet: bool,
    pub verbose: bool,
}

#[allow(dead_code)]
impl OutputOpts {
    pub fn emit_or_human<T: Serialize>(
        &self,
        value: &T,
        human: impl FnOnce() -> String,
    ) -> anyhow::Result<()> {
        if self.json {
            serde_json::to_writer_pretty(io::stdout(), value)?;
            println!();
        } else if !self.quiet {
            println!("{}", human());
        }
        Ok(())
    }

    pub fn print_verbose(&self, msg: &str) {
        if self.verbose && !self.quiet {
            eprintln!("[verbose] {msg}");
        }
    }

    pub fn warn(&self, msg: &str) {
        if !self.quiet {
            let _ = writeln!(io::stderr(), "warning: {msg}");
        }
    }
}
