use std::io;
use wesh::cli::*;

fn main() {
    let entry = cli::ConfigBranch::new(None, ">", "oper");
    
    let mut input = String::new();
    loop {
        entry.print_sig();
        match io::stdin().read_line(&mut input) {
            Err(e) => panic!("{}", e),
            _ => (),
        }

    }
}
