pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn git_revision() -> &'static str {
    option_env!("UMP_BUILD_GIT_REVISION").unwrap_or("unknown")
}

pub fn build_time_utc() -> &'static str {
    option_env!("UMP_BUILD_TIME_UTC").unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use super::{build_time_utc, git_revision, version};

    #[test]
    fn build_info_version_is_not_empty() {
        assert!(!version().is_empty());
    }

    #[test]
    fn build_info_git_revision_is_not_empty() {
        assert!(!git_revision().is_empty());
    }

    #[test]
    fn build_info_build_time_is_not_empty() {
        assert!(!build_time_utc().is_empty());
    }
}
