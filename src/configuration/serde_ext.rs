use secrecy::SecretString;

use serde::Serializer;

/// Redact secrecy values when serialising.
///
/// Intended for use in JSON snapshot tests.
pub fn serialise_secret<S>(_item: &SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("[ SECRET ]")
}

mod tests {
    use secrecy::SecretString;

    use crate::configuration::serde_ext::serialise_secret;

    #[derive(serde::Serialize)]
    pub struct SecretStruct {
        #[serde(serialize_with = "serialise_secret")]
        pub secret: SecretString,
    }

    #[test]
    fn serialise_secret_returns_expected_result() {
        // arrange
        let secret_struct = SecretStruct {
            secret: SecretString::from("abracadra"),
        };

        // act
        let outcome = serde_json::to_string(&secret_struct).unwrap();

        // assert
        assert_eq!(&outcome, r#"{"secret":"[ SECRET ]"}"#);
    }
}
