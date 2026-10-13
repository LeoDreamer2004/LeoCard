use std::fs::File;
use std::io::Read;
use std::mem;
use std::path::Path;

const MAX_ARGUMENT_FILE_BYTES: u64 = 256 * 1024;

pub(super) struct ArgumentFile;

impl ArgumentFile {
    pub(super) fn read(path: &Path, cwd: Option<&Path>) -> Option<Vec<String>> {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            cwd?.join(path)
        };
        let metadata = path.metadata().ok()?;
        if !metadata.is_file() || metadata.len() > MAX_ARGUMENT_FILE_BYTES {
            return None;
        }
        let mut bytes = Vec::new();
        File::open(path)
            .ok()?
            .take(MAX_ARGUMENT_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .ok()?;
        if bytes.len() as u64 > MAX_ARGUMENT_FILE_BYTES {
            return None;
        }
        // Signatures are ASCII; retain them even with platform-encoded paths.
        Some(Self::parse(&String::from_utf8_lossy(&bytes)))
    }

    fn parse(contents: &str) -> Vec<String> {
        let mut arguments = Vec::new();
        let mut argument = String::new();
        let mut quote = None;
        let mut chars = contents.chars().peekable();
        while let Some(character) = chars.next() {
            if quote.is_some() && character == '\\' {
                match chars.next() {
                    Some('\n' | '\r') => {
                        while chars.peek().is_some_and(char::is_ascii_whitespace) {
                            chars.next();
                        }
                    }
                    Some(escaped) => argument.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'f' => '\u{000c}',
                        other => other,
                    }),
                    None => {}
                }
            } else if Some(character) == quote {
                quote = None;
            } else if quote.is_none() && matches!(character, '\'' | '"') {
                quote = Some(character);
            } else if quote.is_none() && character == '#' {
                for character in chars.by_ref() {
                    if matches!(character, '\n' | '\r') {
                        break;
                    }
                }
                Self::finish_argument(&mut arguments, &mut argument);
            } else if character.is_ascii_whitespace() && quote.is_none() {
                Self::finish_argument(&mut arguments, &mut argument);
            } else if !matches!(character, '\n' | '\r') {
                argument.push(character);
            }
        }
        Self::finish_argument(&mut arguments, &mut argument);
        arguments
    }

    fn finish_argument(arguments: &mut Vec<String>, argument: &mut String) {
        if !argument.is_empty() {
            arguments.push(mem::take(argument));
        }
    }
}
