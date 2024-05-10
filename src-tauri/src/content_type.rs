use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref FILE_EXTENSIONS: HashMap<&'static str, &'static str> = [
        ("md", "Markdown"),
        ("c", "C"),
        ("cpp", "C++"),
        ("css", "CSS"),
        ("diff", "Diff"),
        ("erl", "Erlang"),
        ("go", "Go"),
        ("dot", "Graphviz"),
        ("html", "HTML"),
        ("hs", "Haskell"),
        ("java", "Java"),
        ("json", "JSON"),
        ("js", "JavaScript"),
        ("lisp", "Lisp"),
        ("lua", "Lua"),
        ("make", "Makefile"),
        ("matlab", "MATLAB"),
        ("ml", "OCaml"),
        ("m", "Objective-C"),
        ("php", "PHP"),
        ("pl", "Perl"),
        ("py", "Python"),
        ("r", "R"),
        ("re", "Regular Expression"),
        ("rst", "reStructuredText"),
        ("rb", "Ruby"),
        ("rs", "Rust"),
        ("sh", "Shell"),
        ("sql", "SQL"),
        ("xml", "XML"),
        ("yaml", "YAML"),
    ]
    .iter()
    .copied()
    .collect();
}

pub fn process_command(command: &str) -> (String, Option<String>) {
    let parts: Vec<&str> = command.split('|').map(str::trim).collect();
    if let Some(last_part) = parts.last() {
        if let Some(extension) = last_part.strip_prefix('.') {
            if let Some(&content_type) = FILE_EXTENSIONS.get(extension) {
                let new_command = parts[..parts.len() - 1].join(" | ");
                return (new_command, Some(content_type.to_string()));
            }
        }
    }
    (command.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_command_no_extension() {
        let command = "cat example.py | grep something";
        let (new_command, content_type) = process_command(command);
        assert_eq!(new_command, "cat example.py | grep something");
        assert_eq!(content_type, None);
    }

    #[test]
    fn test_process_command_with_extension() {
        let command = "xargs curl -s | example.py | .py";
        let (new_command, content_type) = process_command(command);
        assert_eq!(new_command, "xargs curl -s | example.py");
        assert_eq!(content_type, Some("Python".to_string()));
    }

    #[test]
    fn test_process_command_with_md() {
        let command = "llm | .md";
        let (new_command, content_type) = process_command(command);
        assert_eq!(new_command, "llm");
        assert_eq!(content_type, Some("Markdown".to_string()));
    }
}
