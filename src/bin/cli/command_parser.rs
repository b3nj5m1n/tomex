use clap::{Arg, Command};

pub fn arg_parser_types() -> Vec<Command> {
    vec![
        Command::new("book").about("A book").alias("b"),
        Command::new("series").about("A book series").alias("s"),
        Command::new("review")
            .about("A review of a book")
            .alias("r"),
        Command::new("edition")
            .about("An edition of a book")
            .alias("e"),
        Command::new("edition-review")
            .about("A review of a specific edition of a book")
            .alias("er"),
        Command::new("author").about("An author").alias("a"),
        Command::new("publisher").about("A publisher").alias("pub"),
        Command::new("genre").about("Genres of a book").alias("g"),
        Command::new("mood").about("Mood of a book").alias("m"),
        Command::new("pace").about("Pace of a book"),
        Command::new("language")
            .about("Language of an edition of a book")
            .alias("l"),
        Command::new("progress")
            .about("A progress report for an edition")
            .alias("p"),
    ]
}

pub fn arg_parser() -> Command {
    Command::new("tomex")
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
                .subcommand(
                    Command::new("by_isbn")
                        .about("Add a book by querying OpenLibrary for an ISBN")
                        .alias("isbn"),
                ),
        )
        .subcommand(
            Command::new("edit")
                .about("Edit something (book/review/etc.)")
                .alias("e")
                .alias("update")
                .subcommand_required(true)
                .subcommands(arg_parser_types()),
        )
        .subcommand(
            Command::new("remove")
                .about("Remove something (book/review/etc.)")
                .alias("r")
                .alias("delete")
                .subcommand_required(true)
                .subcommands(arg_parser_types()),
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
                .subcommands(arg_parser_types()),
        )
        .subcommand(Command::new("listen").about("Start a web server for scanning isbn numbers"))
}

pub fn arg_parser_repl() -> Command {
    arg_parser().subcommand(Command::new("exit").about("Exit the repl"))
}

pub fn arg_parser_cli() -> Command {
    arg_parser()
        .subcommand(Command::new("repl").about("Launch a read eval print loop"))
        .subcommand(Command::new("backup").about("Backup the database to JSON"))
        .subcommand(
            Command::new("restore")
                .about("Turn JSON from backup command to new sqlite database")
                .arg(Arg::new("file").required(true)),
        )
        .subcommand(
            Command::new("export")
                .about("Export to a format you can import in goodreads/storygraph/bookwyrm"),
        )
}

pub fn generate_completions() -> Vec<String> {
    let cmd = arg_parser_repl();
    fn add_command(parent_fn_name: &str, cmd: &Command, subcmds: &mut Vec<String>) {
        let fn_name = format!(
            "{parent_fn_name} {cmd_name}",
            parent_fn_name = parent_fn_name,
            cmd_name = cmd.get_name()
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
        add_command("", subcmd, &mut subcmds);
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
