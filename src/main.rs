use std::cell::RefCell;
use std::io;
use std::process;
use std::rc::Rc;
use wesh::cli::cli::StateMessage::*;
use wesh::cli::*;

/// main enters the shell
fn main() {
    let entry = Rc::new(cli::ConfigBranch::new(None, ">", "oper"));
    let config = Rc::new(cli::ConfigBranch::new(
        Some(Rc::downgrade(&entry.clone())),
        "#",
        "conf",
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

    let mut shell_state = cli::ShellState {
        r: cli::Registry::new(),
        cur: entry.clone(),
    };

    shell_state.r.add(entry.clone());
    shell_state.r.add(config.clone());
    shell_state.r.add(exit.clone());
    shell_state.r.add(up.clone());

    drop(entry);
    drop(config);
    drop(exit);
    drop(up);

    loop {
        let mut input = String::new();

        shell_state.cur.print_sig();
        match io::stdin().read_line(&mut input) {
            Err(e) => panic!("{}", e),
            _ => (),
        }

        match shell_state.r.determine_activity(input.trim()) {
            DoNothing => {} // The preferred option
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
