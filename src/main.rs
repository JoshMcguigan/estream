use regex::Regex;
use std::{env, io, io::BufRead};

const VERSION: &str = env!("CARGO_PKG_VERSION");

static FILE_COLON_LINE_COLON_COLUMN: &str =
    r"(?P<file>\S+):(?P<line>[[:digit:]]+):(?P<column>[[:digit:]]+)";
static FILE_COLON_LINE: &str = r"(?P<file>\S+):(?P<line>[[:digit:]]+)";

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some("--version") = args.get(1).map(|s| s.as_str()) {
        println!("v{}", VERSION);
        return;
    }

    let stdin = io::stdin();

    let file_colon_line_colon_column = Regex::new(FILE_COLON_LINE_COLON_COLUMN).unwrap();
    let file_colon_line = Regex::new(FILE_COLON_LINE).unwrap();

    for line in stdin.lock().lines() {
        // For each line of stdin:
        //   - echo the line to stdout
        //   - look for error locations (`test_file.py:15`)
        //      - echo an additional line for any detected error condition

        let line = line.unwrap();
        println!("{}", &line);

        if let Some(captures) = file_colon_line_colon_column.captures(&line) {
            let file = &captures["file"];
            // It is a bit odd that we are unwrapping here, then wrapping with
            // Some below. This is to express the fact that line/column should
            // always be numbers (modulo a programming error), so we want to
            // ensure they can be parsed.
            let line = &captures["line"].parse().unwrap();
            let column = &captures["column"].parse().unwrap();

            let e = ErrorLocation {
                file,
                line: Some(*line),
                column: Some(*column),
            };
            handle_error_line(e);
        } else if let Some(captures) = file_colon_line.captures(&line) {
            let file = &captures["file"];
            // See note above on why we unwrap here only to wrap in Some below.
            let line = &captures["line"].parse().unwrap();

            let e = ErrorLocation {
                file,
                line: Some(*line),
                column: None,
            };
            handle_error_line(e);
        }
    }
}

fn handle_error_line(e: ErrorLocation) {
    // Only print valid file paths. This relies on the
    // working directory of estream matching the working
    // directory of VIM.
    if !std::path::Path::new(e.file).exists() {
        return;
    }

    match e {
        ErrorLocation {
            file,
            line: Some(line),
            column: Some(column),
        } => println!("{}|{}|{}", file, line, column),
        ErrorLocation {
            file,
            line: Some(line),
            column: None,
        } => println!("{}|{}|", file, line),
        ErrorLocation {
            file, line: None, ..
        } => println!("{}||", file),
    }
}

struct ErrorLocation<'a> {
    file: &'a str,
    line: Option<u32>,
    column: Option<u32>,
}

#[cfg(test)]
mod file_colon_line_colon_column {
    use super::FILE_COLON_LINE_COLON_COLUMN;
    use regex::{Captures, Regex};

    fn re(input: &str) -> Option<Captures> {
        Regex::new(FILE_COLON_LINE_COLON_COLUMN)
            .unwrap()
            .captures(input)
    }

    #[test]
    fn simple() {
        let input = "test.txt:20:11";
        let captures = re(input).unwrap();

        assert_eq!("test.txt", &captures["file"]);
        assert_eq!("20", &captures["line"]);
        assert_eq!("11", &captures["column"]);
    }

    #[test]
    fn underscore() {
        let input = "  --> dir/test_underscore.txt:20:11";
        let captures = re(input).unwrap();

        assert_eq!("dir/test_underscore.txt", &captures["file"]);
        assert_eq!("20", &captures["line"]);
        assert_eq!("11", &captures["column"]);
    }

    #[test]
    fn module_no_match() {
        let input = "tests::test_name";
        let captures = re(input);

        assert!(captures.is_none());
    }

    #[test]
    fn missing_file_no_match() {
        let input = " :88:12";
        let captures = re(input);

        assert!(captures.is_none());
    }

    #[test]
    fn missing_column_no_match() {
        // This is python style output, so eventually we want
        // to match this.
        let input = "test.py:88: AssertionError";
        let captures = re(input);

        assert!(captures.is_none());
    }

    #[test]
    fn leading_chars() {
        let input = " --> test.txt:20:11";
        let captures = re(input).unwrap();

        assert_eq!("test.txt", &captures["file"]);
        assert_eq!("20", &captures["line"]);
        assert_eq!("11", &captures["column"]);
    }

    #[test]
    fn trailing_chars() {
        let input = "test.txt:20:11 FAIL";
        let captures = re(input).unwrap();

        assert_eq!("test.txt", &captures["file"]);
        assert_eq!("20", &captures["line"]);
        assert_eq!("11", &captures["column"]);
    }

    #[test]
    fn long_path() {
        let input = " --> in/nested/dir/test.txt:20:11 FAIL";
        let captures = re(input).unwrap();

        assert_eq!("in/nested/dir/test.txt", &captures["file"]);
        assert_eq!("20", &captures["line"]);
        assert_eq!("11", &captures["column"]);
    }
}

#[cfg(test)]
mod file_colon_line {
    use super::FILE_COLON_LINE;
    use regex::{Captures, Regex};

    fn re(input: &str) -> Option<Captures> {
        Regex::new(FILE_COLON_LINE).unwrap().captures(input)
    }

    #[test]
    fn simple() {
        let input = "test.txt:20";
        let captures = re(input).unwrap();

        assert_eq!("test.txt", &captures["file"]);
        assert_eq!("20", &captures["line"]);
    }
}
