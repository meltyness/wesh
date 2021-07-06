use std::io;
use wesh::cli::StateMessage;
use wesh::cli::*;

/// Main drops you into the shell, used as follows:
///
/// # Commands
/// oper - Enter operational mode configuration branch (top-level)
/// conf - Enter configuration mode configuration branch
/// up - Traverse up to the parent configuration branch
/// exit - Immediately leave the shell
fn main() {
    let mut shell_state = cli::initalize_shell();

    loop {
        let mut input = String::new();

        shell_state.cur.print_sig();
        match io::stdin().read_line(&mut input) {
            Err(e) => panic!("{}", e),
            _ => (),
        }

        match shell_state.r.determine_activity(input.trim()) {
            StateMessage::UnknownCommand => {
                println!("Unknown command, `{}`", input.trim());
            }
            StateMessage::StateMove(new_branch) => {
                shell_state.cur = new_branch;
            }
            StateMessage::RunFunction(meta_command) => {
                //println!("Running command {:?}", meta_c);
                (meta_command.action)(&mut shell_state);
            }
        }
    }
}
