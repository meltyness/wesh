use std::io;
use std::process;
use std::rc::Rc;
use wesh::cli::cli::StateMessage::*;
use wesh::cli::*;

/// main enters the shell
fn main() {
    let mut r = cli::Registry::new();
    let entry = Rc::new(cli::ConfigBranch::new(None, ">", "oper"));
    let config = Rc::new(cli::ConfigBranch::new(
        Some(Rc::downgrade(&entry.clone())),
        "#",
        "conf",
    ));
    let exit = Rc::new(cli::MetaCommand::new(
        "exit",
        "Exits the current branch",
        process::exit,
    ));
    let up = Rc::new(cli::MetaCommand::new(
        "up",
        "Exits the current branch",
        process::exit,
    ));

    r.add(entry.clone());
    r.add(config.clone());
    r.add(exit.clone());
    r.add(up.clone());

    let mut cur = entry.clone();

    drop(entry);
    drop(config);
    drop(exit);
    drop(up);

    println!("entry count: {}", Rc::strong_count(&cur));
    loop {
        let mut input = String::new();

        println!("entry count: {}", Rc::strong_count(&cur));
        cur.print_sig();
        match io::stdin().read_line(&mut input) {
            Err(e) => panic!("{}", e),
            _ => (),
        }

        match r.determine_activity(input.trim()) {
            DoNothing => {} // The preferred option
            StateMove(new_br) => {
                cur = new_br;
                println!("Moving to state {:?}", cur);
                println!("entry count: {}", Rc::strong_count(&cur));
            }
            RunFunction(meta_c) => {
                println!("Running command {:?}", meta_c);
                (meta_c.action)(0);
            }
        }
    }
}
