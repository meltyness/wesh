pub mod cli {
    use std::io::Write;
    use std::rc::Rc;
   
    /// ConfigBranches compose the tree-structure of the shell
    ///
    /// Upon entering the shell you are dumped into the entry branch
    ///
    /// The program exits if you issue an "exit" while you are in the entry branch
    pub struct ConfigBranch { 
        parent: Option<Rc<ConfigBranch>>,
        display: Option<String>,
        command_str: Option<String>,
    }

    impl ConfigBranch {
        pub fn print_sig(&self) {
            print!("{} ", self.display.as_ref().unwrap());
            std::io::stdout().flush().unwrap();
        }

        pub fn new(parent: Option<Rc<ConfigBranch>>, display: &str, command_str: &str) -> ConfigBranch{
            ConfigBranch {
                parent: parent,
                display: Some(String::from(display)),
                command_str: Some(String::from(command_str)),
            }
        }
    }

    /// MetaCommands exist regardless of the branch level you're at
    pub struct MetaCommand {
        command_str: Option<String>,
        brief: Option<String>,
    }

    /// Directives are commands
    pub trait Directive { }
    impl Directive for ConfigBranch { }

    /// All Directives must be registered
    pub struct Registry {
        known_directives: Vec<Option<Rc<dyn Directive>>>,
    }

    impl Registry {
        pub fn new() -> Registry {
            Registry {
                known_directives: vec![],
            }
        }

        pub fn add(&mut self, c: Rc<dyn Directive>) {
            self.known_directives.push(Some(c));
        }
    }
}
