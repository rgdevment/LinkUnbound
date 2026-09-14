/// Splitting on spaces is what the field said it did, and it broke the most ordinary argument
/// there is: `--profile-directory="Profile 1"` reached the browser as two, and it opened the
/// default profile instead. Quotes hold a value together, the way every shell and every command
/// line in a browser's own documentation writes them.
#[must_use]
pub fn split(said: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut held: Option<char> = None;
    let mut started = false;

    for c in said.chars() {
        match held {
            Some(quote) if c == quote => held = None,
            Some(_) => word.push(c),
            None if c == '"' || c == '\'' => {
                held = Some(c);
                // An empty pair is an argument: `--flag=""` says something different from
                // `--flag=`, and a browser that reads it can tell them apart.
                started = true;
            }
            None if c.is_whitespace() => {
                if started || !word.is_empty() {
                    out.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            None => word.push(c),
        }
    }
    if started || !word.is_empty() {
        out.push(word);
    }
    out
}

/// What the field shows for what was stored. An argument that carries a space has to come back
/// quoted or the next save would split it again.
#[must_use]
pub fn join(args: &[String]) -> String {
    args.iter()
        .map(|one| {
            if one.is_empty() || one.chars().any(char::is_whitespace) {
                format!("\"{one}\"")
            } else {
                one.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{join, split};

    fn said(raw: &str) -> Vec<String> {
        split(raw)
    }

    /// The report that started this: two people, two browsers, the same answer from both.
    #[test]
    fn a_profile_whose_name_has_a_space_survives_to_the_browser() {
        assert_eq!(
            said(r#"--profile-directory="Profile 1""#),
            vec!["--profile-directory=Profile 1"]
        );
        assert_eq!(
            said(r#"-P "Personal" -no-remote"#),
            vec!["-P", "Personal", "-no-remote"]
        );
    }

    #[test]
    fn the_ordinary_case_is_unchanged() {
        assert_eq!(
            said("--new-window --incognito"),
            vec!["--new-window", "--incognito"]
        );
        assert_eq!(said("   --new-window   "), vec!["--new-window"]);
        assert_eq!(said(""), Vec::<String>::new());
    }

    /// A path with spaces is the other half of the same complaint.
    #[test]
    fn a_quoted_path_is_one_argument() {
        assert_eq!(
            said(r#""C:\Program Files\Sandboxie\Start.exe" /box:Browsers"#),
            vec![r"C:\Program Files\Sandboxie\Start.exe", "/box:Browsers"]
        );
    }

    #[test]
    fn either_quote_holds_a_value_together() {
        assert_eq!(said("--flag='two words'"), vec!["--flag=two words"]);
        assert_eq!(said(r#"--flag="two words""#), vec!["--flag=two words"]);
    }

    /// A quote nobody closed is a typo, and dropping the rest of the line would hide it. What was
    /// written is kept as one argument, where the person can see it and fix it.
    #[test]
    fn a_quote_left_open_keeps_what_follows() {
        assert_eq!(said(r#"--flag="Profile 1"#), vec!["--flag=Profile 1"]);
    }

    #[test]
    fn an_empty_pair_is_still_an_argument() {
        assert_eq!(said(r#"--flag="""#), vec!["--flag="]);
        assert_eq!(said(r#""""#), vec![""]);
    }

    /// What the screen shows has to be what a save reads back, or an argument with a space would
    /// be split the second time it was looked at.
    #[test]
    fn what_is_shown_reads_back_as_what_was_stored() {
        for one in [
            vec!["--profile-directory=Profile 1".to_owned()],
            vec!["-P".to_owned(), "Personal".to_owned()],
            vec!["--new-window".to_owned()],
            vec![String::new()],
            vec![r"C:\Program Files\x\y.exe".to_owned(), "/box:B".to_owned()],
        ] {
            assert_eq!(split(&join(&one)), one, "{one:?}");
        }
    }

    #[test]
    fn nothing_needless_is_quoted() {
        assert_eq!(join(&["--new-window".to_owned()]), "--new-window");
        assert_eq!(
            join(&["--profile-directory=Profile 1".to_owned()]),
            "\"--profile-directory=Profile 1\""
        );
    }
}
