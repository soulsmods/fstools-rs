use std::error::Error;

use clap::{FromArgMatches, Subcommand};
use fstools_dvdbnd::DvdBnd;
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::Action;

pub fn process_input(input: &str, dvd_bnd: &DvdBnd) -> Result<(), Box<dyn Error>> {
    let args = shlex::split(input).ok_or("failed to parse input")?;

    let command = Action::augment_subcommands(clap::Command::new("repl").no_binary_name(true));
    let action = command
        .clone()
        .try_get_matches_from(args)
        .and_then(|matches| Action::from_arg_matches(&matches))
        .map_err(|e| e.to_string())?;

    match action {
        Action::Repl => {
            println!("Already running repl.");
            Ok(())
        }
        action => {
            action.run(dvd_bnd)
        }
    }
}

pub fn begin(dvd_bnd: &DvdBnd) -> Result<(), Box<dyn Error>> {
    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history("history.txt");

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                match process_input(&line, dvd_bnd) {
                    Ok(_) => println!("Command successful"),
                    Err(e) => {
                        println!("{:#?}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
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
