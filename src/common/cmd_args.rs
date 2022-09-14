use ::clap::StructOpt;

use crate::common::Task;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
#[structopt(name = "command")]
pub enum CommandArgs {
    #[structopt(external_subcommand)]
    Cmd(Vec<String>),
}

impl CommandArgs {
    pub fn unpack(self) -> Vec<String> {
        match self {
            CommandArgs::Cmd(cmd) => cmd,
        }
    }

    pub fn into_task(self) -> Task {
        Task::new_split_in_cwd(self.unpack())
    }

    pub fn split_once_at(self, separator: &str) -> (CommandArgs, CommandArgs) {
        let mut first = vec![];
        let mut second = vec![];
        let mut current = &mut first;
        let mut is_first = true;
        for part in self.unpack().drain(..) {
            if is_first && part == separator {
                current = &mut second;
                is_first = false
            } else {
                current.push(part)
            }
        }
        (CommandArgs::Cmd(first), CommandArgs::Cmd(second))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_in_middle() {
        let orig = CommandArgs::Cmd(vec![
            "aaa".to_owned(),
            "----".to_owned(),
            "--".to_owned(),
            "bbb".to_owned(),
            "--".to_owned(),
            "ccc".to_owned(),
        ]);
        let (left, right) = orig.split_once_at("--");
        assert_eq!(left.unpack(), vec!["aaa".to_owned(), "----".to_owned(),]);
        assert_eq!(
            right.unpack(),
            vec!["bbb".to_owned(), "--".to_owned(), "ccc".to_owned(),]
        );
    }
}
