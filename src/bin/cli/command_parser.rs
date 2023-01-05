use clap::Command;

pub fn arg_parser() -> Command {
    Command::new("bokhylle")
        .about("Personal book management")
        .multicall(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("add")
                .about("Add something (book/review/etc.)")
                .subcommand_required(true)
                .subcommand(Command::new("book"))
                .subcommand(Command::new("author")),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove something (book/review/etc.)")
                .subcommand_required(true)
                .subcommand(Command::new("book"))
                .subcommand(Command::new("author")),
        )
}

pub fn arg_parser_cli() -> Command {
    arg_parser().subcommand(Command::new("repl").about("Launch a read eval print loop"))
}

pub fn generate_completions() -> Vec<String> {
    let cmd = arg_parser();
    fn add_command(parent_fn_name: &str, cmd: &Command, subcmds: &mut Vec<String>) {
        let fn_name = format!(
            "{parent_fn_name} {cmd_name}",
            parent_fn_name = parent_fn_name,
            cmd_name = cmd.get_name().to_string()
        )
        .trim()
        .to_string();
        subcmds.push(fn_name.clone());
        for subcmd in cmd.get_subcommands() {
            add_command(&fn_name, subcmd, subcmds);
        }
    }
    let mut subcmds = vec![];
    for subcmd in cmd.get_subcommands() {
        add_command(&"", subcmd, &mut subcmds);
    }
    subcmds.sort();
    subcmds
}

/* pub struct CommandCompleter;

impl Completer for CommandCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<reedline::Suggestion> {
        /* let args = command_parser::arg_parser();
        let command = shlex::split(&command);
        if let None = command {
            println!("Invalid command.");
            return;
        }
        let command = command.unwrap();
        let matches = args.try_get_matches_from(command);
        if let Err(e) = matches {
            println!("{}", e);
            return;
        }
        let matches = matches.unwrap(); */
        let span_line = "test".to_string();
        let span = Span::new(pos - span_line.len(), pos);
        vec![Suggestion {
            value: span_line,
            description: Some("test".into()),
            extra: None,
            span,
            append_whitespace: true,
        }]
    }
} */
