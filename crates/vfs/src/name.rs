use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Name(pub u64);

impl<S: AsRef<Path>> From<S> for Name {
    fn from(value: S) -> Self {
        let value = value.as_ref();
        let prefix = if !value.starts_with("/") {
            Some('/')
        } else {
            None
        };

        let hash = prefix
            .into_iter()
            .chain(
                value
                    .to_string_lossy()
                    .chars()
                    .map(|ch| ch.to_ascii_lowercase()),
            )
            .map(|ch| match ch {
                '\\' => '/',
                _ => ch,
            })
            .fold(0u64, |hash, next| {
                hash.wrapping_mul(0x85).wrapping_add(next as u64)
            });

        Name(hash)
    }
}

#[cfg(test)]
mod test {
    use super::Name;

    #[test]
    pub fn adds_prefix() {
        assert_eq!(Name::from("/path"), Name::from("path"));
    }
}
