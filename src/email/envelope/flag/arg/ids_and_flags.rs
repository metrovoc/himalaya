use clap::Parser;
use email::flag::{Flag, Flags};

/// The ids and/or flags arguments parser.
#[derive(Debug, Parser)]
pub struct IdsAndFlagsArgs {
    /// The list of envelope ids.
    ///
    /// Repeat `--id` for each non-numeric envelope you want to target.
    #[arg(long = "id", short = 'i', value_name = "ID")]
    pub ids: Vec<String>,

    /// The list of flags to apply.
    ///
    /// When `--id` is omitted, plain decimal values keep their legacy
    /// behavior and are interpreted as envelope ids.
    #[arg(value_name = "ID-OR-FLAG", required = true)]
    pub flags: Vec<String>,
}

pub fn into_tuple(args: &IdsAndFlagsArgs) -> (Vec<String>, Flags) {
    let use_legacy_numeric_ids = args.ids.is_empty();
    let mut ids = args.ids.clone();
    let flags = args
        .flags
        .iter()
        .filter_map(|flag| {
            if use_legacy_numeric_ids && flag.parse::<usize>().is_ok() {
                ids.push(flag.clone());
                None
            } else {
                Some(Flag::from(flag.as_str()))
            }
        })
        .collect();

    (ids, flags)
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use email::flag::Flag;

    use super::{into_tuple, IdsAndFlagsArgs};

    #[test]
    fn parses_string_ids_and_flags_without_ambiguity() {
        let args = IdsAndFlagsArgs::try_parse_from([
            "cmd",
            "--id",
            "19be48ac82b25a5b",
            "--id",
            "42",
            "seen",
            "custom-flag",
        ])
        .unwrap();

        let (ids, flags) = into_tuple(&args);

        assert_eq!(ids, vec!["19be48ac82b25a5b", "42"]);
        assert!(flags.contains(&Flag::Seen));
        assert!(flags.contains(&Flag::Custom("custom-flag".into())));
    }

    #[test]
    fn preserves_legacy_numeric_id_parsing_when_no_explicit_id_is_used() {
        let args = IdsAndFlagsArgs::try_parse_from(["cmd", "42", "seen"]).unwrap();

        let (ids, flags) = into_tuple(&args);

        assert_eq!(ids, vec!["42"]);
        assert!(flags.contains(&Flag::Seen));
    }
}
