use crate::{analyze::SeqData, intr::Interpreter, program::Program};

use anyhow::Context;
use clap::Parser;

use std::{fs::File, io::Read};

mod analyze;
mod closure;
mod ctrl_stack;
mod intr;
mod opnd;
pub mod program;
mod runtime;
mod util;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String,

    #[arg(short, long)]
    analyze: bool,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let mut data = Vec::<u8>::new();
    {
        let mut file =
            File::open(&args.file).context(format!("Couldn't open bytecode file {}", args.file))?;
        file.read_to_end(&mut data)?;
        Ok::<_, anyhow::Error>(())
    }
    .context(format!("Couldn't read bytecode file {}", args.file))?;
    let program = Program::load(data.as_slice()).context("Couldn't load program")?;
    if args.analyze {
        println!("{}", SeqData::from_code(program.code));
    } else {
        Interpreter::new(program)?
            .run()
            .context("Running program failed")?;
    }

    Ok(())
}
