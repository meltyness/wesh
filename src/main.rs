use std::io;
use std::process;
use std::rc::Rc;
use wesh::cli::cli::StateMessage::*;
use wesh::cli::*;

/// Main drops you into the shell, used as follows:
///
/// # Commands
/// oper - Enter operational mode configuration branch (top-level)
/// conf - Enter configuration mode configuration branch
/// up - Traverse up to the parent configuration branch
/// exit - Immediately leave the shell
fn main() {
    let entry = Rc::new(cli::ConfigBranch::new(None, ">", "oper", "Enter operational mode"));
    let config = Rc::new(cli::ConfigBranch::new(
        Some(Rc::downgrade(&entry.clone())),
        "#",
        "conf",
        "Enter global Configuration mode",
    ));
    let exit = Rc::new(cli::MetaCommand::new(
        "exit",
        "Exits the current branch",
        Box::new(|_shell| process::exit(0)),
    ));
    let up = Rc::new(cli::MetaCommand::new(
        "up",
        "Exits the current branch",
        Box::new(|shell| match &shell.cur.parent {
            None => process::exit(0),
            Some(b) => shell.cur = b.upgrade().unwrap(),
        }),
    ));
    let csh = Rc::new(cli::MetaCommand::new(
        "?",
        "Requests a copy of the description of all directives",
        Box::new(|shell| for d in &shell.r.known_directives {
            println!("{} - {}", d.get_cmd(), d.get_brief()); 
        }),
    ));

    let mut shell_state = cli::ShellState {
        r: cli::Registry::new(),
        cur: entry.clone(),
    };

    shell_state.r.add(entry.clone());
    shell_state.r.add(config.clone());
    shell_state.r.add(exit.clone());
    shell_state.r.add(up.clone());
    shell_state.r.add(csh.clone());
    drop(entry);
    drop(config);
    drop(exit);
    drop(up);
    drop(csh);

    loop {
        let mut input = String::new();

        shell_state.cur.print_sig();
        match io::stdin().read_line(&mut input) {
            Err(e) => panic!("{}", e),
            _ => (),
        }

        match shell_state.r.determine_activity(input.trim()) {
            UnknownCommand => {
                println!("Unknown command, `{}`", input.trim());
            }
            StateMove(new_br) => {
                shell_state.cur = new_br;
            }
            RunFunction(meta_c) => {
                //println!("Running command {:?}", meta_c);
                (meta_c.action)(&mut shell_state);
            }
        }
    }
}
