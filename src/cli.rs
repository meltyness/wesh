pub mod cli {
    use std::cell::RefMut;
    use std::io::Write;
    use std::rc::{Rc, Weak};

    pub struct ShellState {
        pub r: Registry,
        pub cur: Rc<ConfigBranch>,
    }

    /// ConfigBranches compose the tree-structure of the shell
    ///
    /// Upon entering the shell you are dumped into the entry branch
    ///
    /// The program exits if you issue an "exit" while you are in the entry branch
    #[derive(Debug)]
    pub struct ConfigBranch {
        /// If applicable, the parent Configuration branch
        pub parent: Option<Weak<ConfigBranch>>,
        /// The prefix displayed on the commandline, when in this branch
        pub display: String,
        /// The command-string typed to enter this specific
        pub command_str: String,
    }

    impl ConfigBranch {
        pub fn print_sig(&self) {
            print!("{} ", self.display);
            std::io::stdout().flush().unwrap();
        }

        pub fn new(
            parent: Option<Weak<ConfigBranch>>,
            display: &str,
            command_str: &str,
        ) -> ConfigBranch {
            ConfigBranch {
                parent: parent,
                display: String::from(display),
                command_str: String::from(command_str),
            }
        }
    }

    /// MetaCommands exist regardless of the branch level you're at
    ///
    /// An example is exit defined as follows
    /// command_str: "exit"
    /// brief: "Exits the current branch"
    /// action: actually just exits the program, but needs to make this determination based on
    /// current hierarchical level, which would be gleaned from a borrow from the Register.
    pub struct MetaCommand {
        command_str: String,
        brief: String,
        pub action: Box<dyn Fn(&mut ShellState)>,
    }

    impl MetaCommand {
        pub fn new(
            command_str: &str,
            brief: &str,
            action: Box<dyn Fn(&mut ShellState)>,
        ) -> MetaCommand {
            MetaCommand {
                command_str: String::from(command_str),
                brief: String::from(brief),
                action: action,
            }
        }
    }

    use std::any::Any;
    /// Directives are commands
    pub trait Directive: Any {
        fn get_cmd<'a>(&'a self) -> &'a str;
        fn get_state_message<'a>(&'a self, d: Rc<dyn Any>) -> StateMessage;
        fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
    }

    impl Directive for ConfigBranch {
        fn get_cmd<'a>(&'a self) -> &'a str {
            &*self.command_str
        }
        fn get_state_message<'a>(&'a self, d: Rc<dyn Any>) -> StateMessage {
            StateMessage::StateMove(d.downcast::<ConfigBranch>().unwrap())
        }
        fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
            self
        }
    }

    impl Directive for MetaCommand {
        fn get_cmd<'a>(&'a self) -> &'a str {
            &*self.command_str
        }
        fn get_state_message<'a>(&'a self, d: Rc<dyn Any>) -> StateMessage {
            StateMessage::RunFunction(d.downcast::<MetaCommand>().unwrap())
        }

        fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
            self
        }
    }

    pub enum StateMessage {
        StateMove(Rc<ConfigBranch>),
        RunFunction(Rc<MetaCommand>),
        DoNothing,
    }

    /// All Directives must be registered
    ///
    /// For a given string, the registry has methods to determine what action to take.
    pub struct Registry {
        known_directives: Vec<Rc<dyn Directive>>,
    }

    impl Registry {
        pub fn new() -> Registry {
            Registry {
                known_directives: vec![],
            }
        }

        pub fn add(&mut self, c: Rc<dyn Directive>) {
            self.known_directives.push(c);
        }

        pub fn determine_activity(&self, input: &str) -> StateMessage {
            for d in &self.known_directives {
                if input == d.get_cmd() {
                    let d_a: Rc<dyn Any> = d.clone().as_any_rc();
                    return d.get_state_message(d_a);
                }
            }
            println!("Unknown command, `{}`", input);
            self.dump_commands();
            StateMessage::DoNothing
        }

        pub fn dump_commands(&self) {
            for d in &self.known_directives {
                println!("known command: {}", d.get_cmd());
            }
        }
    }
}
