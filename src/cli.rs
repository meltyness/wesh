pub use self::cli::initalize_shell;
pub use self::cli::StateMessage;

/// The CLI module broadly contains structures organizing the CLI.
pub mod cli {
    use std::io::Write;
    use std::process;
    use std::rc::{Rc, Weak};
    use crate::from_netlink::from_netlink;
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

    /// This initializes the shell, and returns the value to the main CLI loop
    /// The entry config branch is set as operational mode
    ///
    /// | Name                   | Command       | Available Child commands|
    /// |------------------------|---------------|-------------------------|
    /// |Operational Mode        | ```oper ..``` |   ```conf```            |
    /// |Configuration mode      | ```conf ..``` |   None                  |
    /// |Up                      | ```up```      |   None                  |
    /// |Exit                    | ```exit```    |   None                  |
    /// |Context-sensitive help  | ```?```       |   None                  |
    pub fn initalize_shell() -> ShellState {
        let entry = Rc::new(ConfigBranch::new(
            None,
            ">",
            "oper",
            "Enter operational mode",
        ));

        let config = Rc::new(ConfigBranch::new(
            Some(Rc::downgrade(&entry.clone())),
            "#",
            "conf",
            "Enter global Configuration mode",
        ));

        let show = Rc::new(MetaCommand::new(
            "show",
            "Displays configuration information",
            Box::new(|_shell| from_netlink::get_route_table().expect("bad thing"))
            )
        );

        let exit = Rc::new(MetaCommand::new(
            "exit",
            "Exits the current branch",
            Box::new(|_shell| process::exit(0)),
        ));

        let up = Rc::new(MetaCommand::new(
            "up",
            "Exits the current branch",
            Box::new(|shell| match &shell.cur.parent {
                None => process::exit(0),
                Some(b) => shell.cur = b.upgrade().unwrap(),
            }),
        ));

        let csh = Rc::new(MetaCommand::new(
            "?",
            "Requests a copy of the description of all directives",
            Box::new(|shell| {
                for d in &shell.r.known_directives {
                    println!("{} - {}", d.get_cmd(), d.get_brief());
                }
            }),
        ));

        let mut shell_state = ShellState {
            r: Registry::new(),
            cur: entry.clone(),
        };

        shell_state.r.add(entry.clone());
        shell_state.r.add(config.clone());
        shell_state.r.add(show.clone());
        shell_state.r.add(exit.clone());
        shell_state.r.add(up.clone());
        shell_state.r.add(csh.clone());

        shell_state
    }
}
