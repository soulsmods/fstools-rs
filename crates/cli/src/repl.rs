use std::{error::Error, fs};

use clap::{FromArgMatches, Subcommand};
use directories::ProjectDirs;
use fstools_dvdbnd::DvdBnd;
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::{Action, GameType};

pub fn process_input(
    input: &str,
    dvd_bnd: &DvdBnd,
    game_type: &GameType,
) -> Result<(), Box<dyn Error>> {
    let args = shlex::split(input).ok_or("failed to parse input")?;
    let command = Action::augment_subcommands(clap::Command::new("").no_binary_name(true))
        .mut_subcommand("repl", |cmd| cmd.hide(true))
        .color(clap::ColorChoice::Always);

    let action = command
        .clone()
        .try_get_matches_from(args)
        .and_then(|matches| Action::from_arg_matches(&matches));

    match action {
        Ok(Action::Repl) => {
            println!("Already running repl.");
            Ok(())
        }
        Ok(action) => action.run(dvd_bnd, game_type),
        Err(e) => Ok(e.print()?),
    }
}

pub fn begin(dvd_bnd: &DvdBnd, game_type: &GameType) -> Result<(), Box<dyn Error>> {
    let mut rl = DefaultEditor::new()?;
    let dirs = ProjectDirs::from("io.github", "soulsmods", "fstools_cli");
    let history_path = dirs.map(|project_dirs| project_dirs.data_dir().join("history.txt"));

    if let Some(history_path) = history_path.as_ref() {
        let _ = fs::create_dir_all(history_path.parent().expect("invalid data dir"));
        let _ = rl.load_history(history_path);
    }

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                match process_input(&line, dvd_bnd, game_type) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{}", e);
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

        if let Some(history_path) = history_path.as_ref() {
            rl.save_history(history_path)?;
        }
    }

    Ok(())
}
