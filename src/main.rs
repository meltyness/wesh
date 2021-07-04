use std::io;
use std::rc::Rc;
use wesh::cli::*;

/// main enters the shell
fn main() {
    let mut r = cli::Registry::new(); 
    let entry = Rc::new(cli::ConfigBranch::new(None, ">", "oper"));
    let exit = Rc::new(cli::MetaCommand::new(None, "exit")); 
    let cur = entry.clone()

    r.add(entry.clone());
    let mut input = String::new();
    loop {
        entry.print_sig();
        match io::stdin().read_line(&mut input) {
            Err(e) => panic!("{}", e),
            _ => (),
        }

    }
}
