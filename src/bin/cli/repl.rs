use reedline::{
    ColumnarMenu, DefaultCompleter, Emacs, ExampleHighlighter, FileBackedHistory, KeyCode,
    KeyModifiers, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};

use crate::prompt::BokhyllePrompt;

pub struct Repl {
    reedline: Reedline,
    prompt: BokhyllePrompt,
}

impl Repl {
    pub fn new(commands: Vec<String>) -> Self {
        let history = Box::new(
            FileBackedHistory::with_file(usize::MAX - 1, "history.txt".into())
                .expect("Error configuring history with file"),
        );

        let completer = Box::new(DefaultCompleter::new_with_wordlen(commands.clone(), 1));

        let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));

        let mut keybindings = reedline::default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu("completion_menu".to_string()),
                ReedlineEvent::MenuNext,
            ]),
        );

        let edit_mode = Box::new(Emacs::new(keybindings));

        let prompt = BokhyllePrompt {};

        let line_editor = Reedline::create()
            .with_history(history)
            .with_highlighter(Box::new(ExampleHighlighter::new(commands)))
            .with_completer(completer)
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_edit_mode(edit_mode);

        Repl {
            reedline: line_editor,
            prompt,
        }
    }
    pub fn read_line(&mut self) -> anyhow::Result<Signal> {
        Ok(self.reedline.read_line(&self.prompt)?)
    }
}
