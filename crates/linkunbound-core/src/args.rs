/// Splitting on spaces is what the field said it did, and it broke the most ordinary argument
/// there is: `--profile-directory="Profile 1"` reached the browser as two, and it opened the
/// default profile instead. Double quotes hold a value together, the way Windows writes them, and
/// a doubled quote inside them is one quote — also the way Windows writes it.
///
/// An apostrophe is a letter here, not a quote. `O'Brien` is a folder people have and `Martin's`
/// is a profile people name, and treating either as a delimiter eats the rest of the word — which
/// is the same bug this exists to fix, wearing a different face.
#[must_use]
pub fn split(said: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut started = false;
    let mut letters = said.chars().peekable();

    while let Some(c) = letters.next() {
        match c {
            '"' if quoted && letters.peek() == Some(&'"') => {
                letters.next();
                word.push('"');
            }
            '"' => {
                quoted = !quoted;
                // An empty pair is an argument: `--flag=""` says something different from
                // `--flag=`, and a browser that reads it can tell them apart.
                started = true;
            }
            _ if !quoted && c.is_whitespace() => {
                if started || !word.is_empty() {
                    out.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            _ => word.push(c),
        }
    }
    if started || !word.is_empty() {
        out.push(word);
    }
    out
}

/// What the field shows for what was stored. An argument that carries a space has to come back
/// quoted or the next save would split it again — and so does one that carries a quote, or
/// reopening the settings and saving without touching anything would quietly rewrite it.
#[must_use]
pub fn join(args: &[String]) -> String {
    args.iter()
        .map(|one| {
            if one.is_empty() || one.contains('"') || one.chars().any(char::is_whitespace) {
                format!("\"{}\"", one.replace('"', "\"\""))
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

    /// An apostrophe is a letter. Reading it as a quote ate the rest of the word, which is the
    /// very bug this module was written to fix.
    #[test]
    fn an_apostrophe_is_part_of_the_word() {
        assert_eq!(
            said("--profile-directory=Martin's"),
            vec!["--profile-directory=Martin's"]
        );
        assert_eq!(
            said(r"--path=C:\Users\O'Brien\AppData"),
            vec![r"--path=C:\Users\O'Brien\AppData"]
        );
        assert_eq!(
            said("--profile-directory=Martin's Perfil"),
            vec!["--profile-directory=Martin's", "Perfil"],
            "it separates on the space, like any other word"
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
        assert_eq!(said("\t--new-window\n--foo"), vec!["--new-window", "--foo"]);
    }

    /// A path with spaces is the other half of the same complaint.
    #[test]
    fn a_quoted_path_is_one_argument() {
        assert_eq!(
            said(r#""C:\Program Files\Sandboxie\Start.exe" /box:Browsers"#),
            vec![r"C:\Program Files\Sandboxie\Start.exe", "/box:Browsers"]
        );
    }

    /// Windows writes a literal quote as two, and so does this.
    #[test]
    fn a_doubled_quote_inside_a_value_is_one_quote() {
        assert_eq!(
            said(r#""--user-agent=Mozilla ""5.0""""#),
            vec![r#"--user-agent=Mozilla "5.0""#]
        );
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

    /// What the screen shows has to be what a save reads back, or an argument would be rewritten
    /// every time somebody opened the settings and saved without touching anything. The values
    /// here are the ones that used to break it: quotes, apostrophes, and both at once.
    #[test]
    fn what_is_shown_reads_back_as_what_was_stored() {
        for one in [
            vec!["--profile-directory=Profile 1".to_owned()],
            vec!["-P".to_owned(), "Personal".to_owned()],
            vec!["--new-window".to_owned()],
            vec![String::new()],
            vec![r"C:\Program Files\x\y.exe".to_owned(), "/box:B".to_owned()],
            vec!["it's".to_owned()],
            vec![r"--path=C:\Users\O'Brien\x".to_owned()],
            vec![r#"--flag="quoted""#.to_owned()],
            vec![r#"say "hi" there"#.to_owned()],
            vec!["\"".to_owned()],
            vec![r#"it's "both""#.to_owned()],
            vec![r"--path=C:\Program Files\".to_owned()],
        ] {
            assert_eq!(split(&join(&one)), one, "{one:?}");
        }
    }

    #[test]
    fn nothing_needless_is_quoted() {
        assert_eq!(join(&["--new-window".to_owned()]), "--new-window");
        assert_eq!(
            join(&["it's".to_owned()]),
            "it's",
            "an apostrophe needs nothing"
        );
        assert_eq!(
            join(&["--profile-directory=Profile 1".to_owned()]),
            "\"--profile-directory=Profile 1\""
        );
    }

    #[test]
    fn an_empty_list_and_an_empty_line_are_the_same_thing() {
        assert_eq!(join(&[]), "");
        assert_eq!(split(""), Vec::<String>::new());
    }
}
