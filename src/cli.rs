/// The CLI module broadly contains structures organizing the CLI.
pub mod cli {
    use std::io::Write;
    use std::rc::{Rc, Weak};

    /// ConfigBranches compose the tree-structure of the shell
    ///
    /// Upon entering the shell you are dumped into the entry branch
    ///
    /// The program exits if you issue an "exit" while you are in the entry branch
    #[derive(Debug)]
    pub struct ConfigBranch {
        pub parent: Option<Weak<ConfigBranch>>,
        pub display: String,
        pub command_str: String,
        pub brief: String,
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
            brief: &str,
        ) -> ConfigBranch {
            ConfigBranch {
                parent: parent,
                display: String::from(display),
                command_str: String::from(command_str),
                brief: String::from(brief),
            }
        }
    }

    /// MetaCommands exist regardless of the branch level you're at
    ///
    /// # Example
    /// command_str: "exit"
    /// brief: "Exits the current branch"
    /// action: receives a closure that, regardless of ShellState, terminates the process.
    /// ```
    /// { |_shell| process::exit(0) }
    /// ```
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
    /// Both MetaCommands and ConfigBranches may be used as directives to the shell.
    ///
    /// This provides a common interface for the Registry to determine command
    /// applicability, and report back to the main thread.
    pub trait Directive: Any {
        fn get_cmd<'a>(&'a self) -> &'a str;
        fn get_state_message<'a>(&'a self, d: Rc<dyn Any>) -> StateMessage;
        fn get_brief<'a>(&'a self) -> &'a str;
        fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
    }

    impl Directive for ConfigBranch {
        fn get_cmd<'a>(&'a self) -> &'a str {
            &*self.command_str
        }

        fn get_state_message<'a>(&'a self, d: Rc<dyn Any>) -> StateMessage {
            StateMessage::StateMove(d.downcast::<ConfigBranch>().unwrap())
        }

        fn get_brief<'a>(&'a self) -> &'a str {
            &*self.brief
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

        fn get_brief<'a>(&'a self) -> &'a str {
            &*self.brief
        }

        fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
            self
        }
    }

    /// This enum provides separation between prescribed action, and type.
    pub enum StateMessage {
        StateMove(Rc<ConfigBranch>),
        RunFunction(Rc<MetaCommand>),
        UnknownCommand,
    }

    /// All Directives must be registered
    ///
    /// For a given string, the registry has methods to determine what action to take.
    pub struct Registry {
        pub known_directives: Vec<Rc<dyn Directive>>,
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
            StateMessage::UnknownCommand
        }
    }

    /// The ShellState contains details about the Shell as presented to the user.
    pub struct ShellState {
        pub r: Registry,
        pub cur: Rc<ConfigBranch>,
    }
}
