use std::str::FromStr;

/// git context
///
/// docker style urls see: https://docs.docker.com/engine/reference/commandline/build/
#[derive(Debug, Clone)]
pub struct GitContext {
    /// git repository url
    pub remote: String,
    /// git ref, default is master
    pub git_ref: String,
    /// sub dir, default is empty
    pub sub_dir: String,
}

impl FromStr for GitContext {
    type Err = anyhow::Error;

    /// very simple parsing, just support docker style urls
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (remote, fragment) = s.split_once('#').unwrap_or((s, ""));
        let (git_ref, sub_dir) = fragment.split_once(':').unwrap_or((fragment, ""));

        Ok(GitContext {
            remote: remote.to_string(),
            git_ref: git_ref.to_string(),
            sub_dir: sub_dir.to_string(),
        })
    }
}

impl ToString for GitContext {
    fn to_string(&self) -> String {
        match (
            self.remote.as_str(),
            self.git_ref.as_str(),
            self.sub_dir.as_str(),
        ) {
            (remote, "", "") => remote.to_owned(),
            (remote, git_ref, "") => format!("{}#{}", remote, git_ref),
            (remote, git_ref, sub_dir) => format!("{}#{}:{}", remote, git_ref, sub_dir),
        }
    }
}

impl From<&GitContext> for String {
    fn from(value: &GitContext) -> Self {
        value.to_string()
    }
}

impl From<GitContext> for String {
    fn from(value: GitContext) -> Self {
        (&value).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_git_url_test() {
        let url = "gosh://0:230230293XXXXXXXXXXXX/docker/docker.git#v1.13.0:src/dir";
        let ctx: GitContext = url.parse().unwrap();

        assert_eq!(
            ctx.remote,
            "gosh://0:230230293XXXXXXXXXXXX/docker/docker.git"
        );
        assert_eq!(ctx.git_ref, "v1.13.0");
        assert_eq!(ctx.sub_dir, "src/dir");

        assert_eq!(String::from(&ctx), url);
        assert_eq!(String::from(ctx), url);
    }

    #[test]
    fn docker_git_url_test2() {
        let url = "gosh://0:230230293XXXXXXXXXXXX/docker/docker.git#ref/test";
        let ctx: GitContext = url.parse().unwrap();

        assert_eq!(
            ctx.remote,
            "gosh://0:230230293XXXXXXXXXXXX/docker/docker.git"
        );
        assert_eq!(ctx.git_ref, "ref/test");
        assert_eq!(ctx.sub_dir, "");

        assert_eq!(String::from(ctx), url);
    }

    #[test]
    fn docker_git_url_test3() {
        let url = "gosh://0:230230293XXXXXXXXXXXX/docker/docker.git#:dir/dir";
        let ctx: GitContext = url.parse().unwrap();

        assert_eq!(
            ctx.remote,
            "gosh://0:230230293XXXXXXXXXXXX/docker/docker.git"
        );
        assert_eq!(ctx.git_ref, "");
        assert_eq!(ctx.sub_dir, "dir/dir");

        assert_eq!(String::from(ctx), url);
    }
}
