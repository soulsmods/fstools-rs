use std::error::Error;

use clap::{CommandFactory, FromArgMatches, Subcommand};
use fstools_dvdbnd::DvdBnd;
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::{Action, Cli};

pub fn begin(dvd_bnd: &DvdBnd) -> Result<(), Box<dyn Error>> {
    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history("history.txt");

    let command = Action::augment_subcommands(clap::Command::new("repl").no_binary_name(true));

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let args = shlex::split(&line).ok_or("failed to parse input")?;

                let action = command
                    .clone()
                    .try_get_matches_from(args)
                    .and_then(|matches| Action::from_arg_matches(&matches))
                    .map_err(|e| e.to_string());

                match action {
                    Ok(Action::Repl) => {
                        println!("Running repl inside repl doesn't make sense");
                        continue;
                    }
                    Ok(action) => {
                        let _ = action.run(dvd_bnd);
                    }
                    Err(e) => println!("{}", e),
                };
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }

        rl.save_history("history.txt")?;
    }

    Ok(())
}
