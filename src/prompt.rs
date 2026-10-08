//! Interactive confirmation, shared by the destructive commands (`alias remove`,
//! `delete-bucket`, `delete-file`, `sync --delete`).

use std::io::{self, BufRead, Write};

use crate::error::MyS3Error;

/// Asks `question` on stderr and reads the answer on stdin.
///
/// Only `y` or `yes` (in any case) confirm. Anything else, an empty answer or
/// the end of the input (e.g. no terminal) refuses.
pub fn confirm(question: &str) -> Result<bool, MyS3Error> {
    confirm_with(question, &mut io::stdin().lock(), &mut io::stderr()).map_err(MyS3Error::InputRead)
}

/// Same as [`confirm`], reading the answer from `input` and writing the
/// question to `output`, so that it can be tested.
pub fn confirm_with(
    question: &str,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> io::Result<bool> {
    write!(output, "{question} [y/N] ")?;
    output.flush()?;

    let mut answer = String::new();
    input.read_line(&mut answer)?;
    Ok(matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer to `Delete it?` when the user types `typed`, and what was shown.
    fn ask(typed: &str) -> (bool, String) {
        let mut output = Vec::new();
        let confirmed = confirm_with("Delete it?", &mut typed.as_bytes(), &mut output).unwrap();
        (confirmed, String::from_utf8(output).unwrap())
    }

    #[test]
    fn question_shows_the_default_answer() {
        assert_eq!(ask("y\n").1, "Delete it? [y/N] ");
    }

    #[test]
    fn y_and_yes_confirm_in_any_case() {
        for typed in ["y\n", "yes\n", "Y\n", "YES\n", "  yes  \n", "y"] {
            assert!(ask(typed).0, "{typed:?} should confirm");
        }
    }

    #[test]
    fn anything_else_refuses() {
        for typed in ["n\n", "no\n", "\n", "yep\n", "o\n", ""] {
            assert!(!ask(typed).0, "{typed:?} should refuse");
        }
    }
}
