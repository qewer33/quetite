use std::{borrow::Cow, cell::RefCell, rc::Rc};

use crate::{
    evaluator::{
        Evaluator,
        env::EnvPtr,
        loader::{Loader, LoaderPtr},
        natives::Natives,
        resolver::Resolver,
    },
    lexer::{Lexer, cursor::Cursor},
    parser::Parser,
    reporter::Reporter,
    src::Src,
};

use colored::Colorize;
use crossterm::event::{KeyCode, KeyModifiers};
use minus::{Pager, page_all};
use reedline::{
    DefaultPrompt, DefaultPromptSegment, Emacs, Highlighter, Prompt, Reedline, ReedlineEvent,
    Signal, StyledText, default_emacs_keybindings,
};
use termimad::{
    Alignment, MadSkin, StyledChar,
    crossterm::style::{Attribute, Color},
};

pub struct Repl {
    globals: EnvPtr,
    loader: LoaderPtr,
    src: Src,
    help: Option<HelpIndex>,
    api_help: Option<HelpIndex>,
}

impl Repl {
    pub fn new() -> Self {
        let globals = Natives::get_natives();
        let help = HelpIndex::from_str(include_str!("../REFERENCE.md"));
        let api_help = HelpIndex::from_str(include_str!("../API.md"));

        Self {
            globals,
            loader: Rc::new(RefCell::new(Loader::default())),
            src: Src::repl("<repl>"),
            help,
            api_help,
        }
    }

    pub fn run(&mut self) {
        // setup reedline
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(KeyModifiers::SHIFT, KeyCode::Enter, ReedlineEvent::Enter);
        let edit_mode = Box::new(Emacs::new(keybindings));
        let mut line_editor = Reedline::create()
            .with_edit_mode(edit_mode)
            .with_highlighter(Box::new(QteHighlighter::default()));
        let prompt = QtePrompt::new();

        // welcome text
        println!(
            "Welcome to the {} shell! Type '{}' for more info or '{}' to exit the shell. Use '{}' to create new lines without sending the prompt.",
            "Quetite".yellow(),
            "help".blue(),
            "exit".red(),
            "Alt+Enter".blue()
        );

        loop {
            let sig = line_editor.read_line(&prompt);
            match sig {
                Ok(Signal::Success(mut input)) => {
                    input = input.trim().to_string();
                    if input.is_empty() {
                        continue;
                    }
                    if self.handle_meta(&input) {
                        continue;
                    }

                    // append to session source and capture starting line for accurate cursors
                    let start_line = self.src.append_chunk(&input);

                    // compile & eval input
                    self.compile_chunk(start_line, &input);

                    if self.src.ast.is_some() {
                        let mut evaluator = Evaluator::with_state(
                            &self.src,
                            self.globals.clone(),
                            self.loader.clone(),
                        );
                        match evaluator.eval_with_result() {
                            Ok(res) => {
                                self.globals = evaluator.env;
                                if let Some(val) = res {
                                    println!("{}", val);
                                }
                            }
                            Err(_) => {
                                // error already reported
                            }
                        }
                    }
                }
                Ok(Signal::CtrlC) => {
                    break;
                }
                _ => {}
            }
        }
    }

    // lex -> parse -> resolve
    fn compile_chunk(&mut self, start_line: usize, chunk: &str) {
        // clear previous compile artifacts
        self.src.tokens = None;
        self.src.ast = None;

        // prepare cursor offset so tokens/AST carry absolute lines for reporter
        let mut cursor = Cursor::new();
        cursor.line = start_line;

        let mut lexer = Lexer::with_cursor(chunk.to_string(), cursor);
        let lex_out = lexer.tokenize();
        self.src.tokens = match lex_out.tokens {
            Some(toks) => Some(toks),
            None => {
                if let Some(errs) = lex_out.errors {
                    for err in errs.iter() {
                        Reporter::lex_err_at(err, &self.src);
                    }
                }
                return;
            }
        };

        let mut parser = Parser::new(&self.src);
        let parser_out = parser.parse();
        self.src.ast = match parser_out.ast {
            Some(ast) => {
                if parser_out.warning_count > 0 {
                    Reporter::warning(
                        format!("parser exited with {} warnings", parser_out.warning_count)
                            .as_str(),
                    );
                }
                Some(ast)
            }
            None => return,
        };

        let mut resolver = Resolver::new(&self.src);
        let resolver_out = resolver.resolve();
        self.src.ast = match resolver_out.ast {
            Some(ast) => {
                if resolver_out.warning_count > 0 {
                    Reporter::warning(
                        format!(
                            "resolver exited with {} warnings",
                            resolver_out.warning_count
                        )
                        .as_str(),
                    );
                }
                Some(ast)
            }
            None => return,
        };
    }

    fn handle_meta(&self, input: &str) -> bool {
        if input.eq_ignore_ascii_case("exit") {
            std::process::exit(0);
        }

        if !input.to_lowercase().starts_with("help") {
            return false;
        }

        let parts: Vec<&str> = input.trim_start_matches(':').split_whitespace().collect();

        if parts.len() == 1 {
            println!(
                "The {} command lets you interactively explore the {} docs.",
                "help".blue(),
                "Quetite Language and API Reference".yellow()
            );
            println!();
            println!("  help ref               - list language reference sections");
            println!("  help ref [section|num] - read a language reference section");
            println!("  help api               - list stdlib API reference sections");
            println!("  help api [topic|num]   - read an stdlib API reference section");
            println!();
            println!("Example Usage:");
            println!("  help ref");
            println!("  help ref Type System");
            println!("  help api 2.3");
            return true;
        }

        match parts[1].to_lowercase().as_str() {
            "ref" => {
                if self.help.is_none() {
                    println!("reference help unavailable (REFERENCE.md missing)");
                    return true;
                }
                let h = self.help.as_ref().unwrap();
                if parts.len() == 2 {
                    println!("Quetite Language Reference");
                    println!();
                    h.print_topics();
                } else {
                    let term = parts[2..].join(" ");
                    h.show_section(&term);
                }
                true
            }
            "api" => {
                if self.api_help.is_none() {
                    println!("API help unavailable (API.md missing)");
                    return true;
                }
                let h = self.api_help.as_ref().unwrap();
                if parts.len() == 2 {
                    println!("Quetite API Reference");
                    println!();
                    h.print_topics();
                } else {
                    let term = parts[2..].join(" ");
                    h.show_section(&term);
                }
                true
            }
            _ => {
                println!("unknown help topic. Use 'help ref' or 'help api'.");
                true
            }
        }
    }
}

#[derive(Default)]
struct QteHighlighter;

impl Highlighter for QteHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut text = StyledText::new();
        text.push((nu_ansi_term::Style::new(), line.to_string()));
        text
    }
}

struct QtePrompt {
    inner: DefaultPrompt,
}

impl QtePrompt {
    fn new() -> Self {
        Self {
            inner: DefaultPrompt::new(
                DefaultPromptSegment::Basic(format!("{} ", "qte".yellow())),
                DefaultPromptSegment::CurrentDateTime,
            ),
        }
    }
}

impl Prompt for QtePrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        self.inner.render_prompt_left()
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        self.inner.render_prompt_right()
    }

    fn render_prompt_indicator(&self, edit_mode: reedline::PromptEditMode) -> Cow<'_, str> {
        self.inner.render_prompt_indicator(edit_mode)
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("::::: ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> Cow<'_, str> {
        self.inner
            .render_prompt_history_search_indicator(history_search)
    }
}

#[derive(Debug)]
struct Section {
    title: String,
    level: usize,
    start: usize,
    end: usize,
    number: String,
}

struct HelpIndex {
    lines: Vec<String>,
    sections: Vec<Section>,
}

impl HelpIndex {
    fn from_str(text: &str) -> Option<Self> {
        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        let mut sections: Vec<Section> = Vec::new();
        let mut in_code_block = false;
        let mut counters: Vec<usize> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count();
                if trimmed.chars().nth(level) != Some(' ') {
                    continue;
                }
                if level == 1 {
                    continue;
                }
                while counters.len() < level {
                    counters.push(0);
                }
                counters.truncate(level);
                if let Some(last) = counters.last_mut() {
                    *last += 1;
                }
                let number = counters
                    .iter()
                    .skip(1)
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(".");
                let title = trimmed.trim_start_matches('#').trim().to_string();
                sections.push(Section {
                    title,
                    level,
                    start: i,
                    end: lines.len(),
                    number,
                });
            }
        }

        for idx in 0..sections.len() {
            let level = sections[idx].level;
            let start = sections[idx].start;
            let mut end = lines.len();
            for next in (idx + 1)..sections.len() {
                if sections[next].level <= level {
                    end = sections[next].start;
                    break;
                }
            }
            sections[idx].end = end.max(start + 1);
        }

        Some(Self { lines, sections })
    }

    fn print_topics(&self) {
        for sec in self.sections.iter().filter(|s| s.level <= 3) {
            let indent = "  ".repeat(sec.level.saturating_sub(2));
            println!("{}{} {}", indent, sec.number, sec.title);
        }
    }

    fn show_section(&self, term: &str) {
        println!();
        let needle = term.to_lowercase();
        if let Some(sec) = self
            .sections
            .iter()
            .find(|s| s.title.to_lowercase().contains(&needle) || s.number == term)
        {
            let section_text = self.lines[sec.start..sec.end].join("\n");
            let skin = make_skin();
            let rendered = render_with_skin(&skin, &section_text);
            page_output(&rendered);
        } else {
            println!("Invalid help section");
        }
    }
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.paragraph.set_fg(Color::Reset);

    let header_colors = [33, 32, 31];
    for (i, col) in header_colors.iter().enumerate() {
        if let Some(h) = skin.headers.get_mut(i) {
            h.compound_style.set_fg(Color::AnsiValue(*col));
            h.compound_style.add_attr(Attribute::Bold);
            h.align = Alignment::Left;
        }
    }

    skin.italic.set_fg(Color::AnsiValue(109));
    skin.italic.add_attr(Attribute::Italic);

    skin.bold.set_fg(Color::AnsiValue(15));
    skin.bold.add_attr(Attribute::Bold);

    skin.inline_code.set_fg(Color::AnsiValue(85));
    skin.inline_code.set_bg(Color::AnsiValue(242));
    skin.code_block.compound_style.set_fg(Color::AnsiValue(85));
    skin.code_block.compound_style.set_bg(Color::AnsiValue(242));

    skin.bullet = StyledChar::from_fg_char(Color::AnsiValue(45), '•');
    skin.quote_mark = StyledChar::from_fg_char(Color::AnsiValue(109), '▌');

    skin
}

fn render_with_skin(skin: &MadSkin, text: &str) -> String {
    let mut buf: Vec<u8> = Vec::new();
    let _ = skin.write_text_on(&mut buf, text);
    String::from_utf8(buf).unwrap_or_else(|_| text.to_string())
}

fn page_output(text: &str) {
    let pager = Pager::new();
    let _ = pager.set_exit_strategy(minus::ExitStrategy::PagerQuit);
    if pager.set_text(text).is_err() || page_all(pager).is_err() {
        print!("{text}");
    }
}
