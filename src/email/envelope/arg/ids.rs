use clap::Parser;

/// The envelope id argument parser.
#[derive(Debug, Parser)]
pub struct EnvelopeIdArg {
    /// The envelope id.
    #[arg(value_name = "ID", required = true)]
    pub id: String,
}

/// The envelopes ids arguments parser.
#[derive(Debug, Parser)]
pub struct EnvelopeIdsArgs {
    /// The list of envelopes ids.
    #[arg(value_name = "ID", required = true)]
    pub ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{EnvelopeIdArg, EnvelopeIdsArgs};

    #[test]
    fn parses_arbitrary_single_envelope_ids() {
        let args = EnvelopeIdArg::try_parse_from(["cmd", "19be48ac82b25a5b"]).unwrap();
        assert_eq!(args.id, "19be48ac82b25a5b");
    }

    #[test]
    fn parses_multiple_mixed_envelope_ids() {
        let args = EnvelopeIdsArgs::try_parse_from(["cmd", "19be48ac82b25a5b", "42"]).unwrap();
        assert_eq!(args.ids, vec!["19be48ac82b25a5b", "42"]);
    }
}
