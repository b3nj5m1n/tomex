use clap::{ArgAction, Command};

pub fn arg_parser_types() -> Vec<Command> {
    vec![Command::new("book"), Command::new("author")]
}

pub fn arg_parser() -> Command {
    Command::new("bokhylle")
        .about("Personal book management")
        .multicall(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("add")
                .about("Add something (book/review/etc.)")
                .alias("a")
                .alias("insert")
                .subcommand_required(true)
                .subcommands(arg_parser_types())
        )
        .subcommand(
            Command::new("edit")
                .about("Edit something (book/review/etc.)")
                .alias("e")
                .alias("update")
                .subcommand_required(true)
                .subcommands(arg_parser_types())
        )
        .subcommand(
            Command::new("remove")
                .about("Remove something (book/review/etc.)")
                .alias("r")
                .alias("delete")
                .subcommand_required(true)
                .subcommands(arg_parser_types())
        )
        .subcommand(
            Command::new("query")
                .about("Get existing records in database")
                .alias("q")
                .alias("get")
                .arg(
                    clap::Arg::new("all")
                        .global(true)
                        .required(false)
                        .num_args(0)
                        .short('a')
                        .long("all")
                        .help("Display all records in database"),
                )
                .arg(
                    clap::Arg::new("interactive")
                        .global(true)
                        .required(false)
                        .num_args(0)
                        .short('i')
                        .long("interactive")
                        .help("Launch an interactive search"),
                )
                .arg(
                    clap::Arg::new("uuid")
                        .global(true)
                        .required(false)
                        .num_args(1)
                        .short('u')
                        .long("uuid")
                        .help("Get record by uuid"),
                )
                .subcommand_required(true)
                .subcommands(arg_parser_types())
        )
}

pub fn arg_parser_repl() -> Command {
    arg_parser().subcommand(Command::new("exit").about("Exit the repl"))
}

pub fn arg_parser_cli() -> Command {
    arg_parser().subcommand(Command::new("repl").about("Launch a read eval print loop"))
}

pub fn generate_completions() -> Vec<String> {
    let cmd = arg_parser_repl();
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
