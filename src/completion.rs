use rustyline::{
    Context, Helper,
    completion::{Completer, Pair},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};
use std::result::Result;
pub struct CommandCompleterHelper {
    pub programs: Vec<&'static str>,
}

impl Completer for CommandCompleterHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), rustyline::error::ReadlineError> {
        let prefix = &line[..pos];
        if prefix.contains(' ') || line.is_empty() {
            return Ok((0, Vec::new()));
        }

        let matches = self
            .programs
            .iter()
            .filter(|p| p.starts_with(prefix))
            .map(|cmd| Pair {
                display: cmd.to_string(),
                replacement: format!("{} ", cmd),
            })
            .collect::<Vec<Pair>>();
        Ok((0, matches))
    }
}

impl Hinter for CommandCompleterHelper {
    type Hint = String;
}
impl Highlighter for CommandCompleterHelper {}
impl Validator for CommandCompleterHelper {}
impl Helper for CommandCompleterHelper {}
