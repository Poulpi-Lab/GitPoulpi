use super::*;

#[test]
fn remote_urls_become_the_repositorys_web_page() {
    for (url, want) in [
        (
            "git@github.com:owner/repo.git",
            "https://github.com/owner/repo",
        ),
        ("git@github.com:owner/repo", "https://github.com/owner/repo"),
        (
            "ssh://git@github.com/owner/repo.git",
            "https://github.com/owner/repo",
        ),
        (
            "ssh://git@gitlab.example.com:2222/grp/sub/repo.git",
            "https://gitlab.example.com/grp/sub/repo",
        ),
        (
            "https://github.com/owner/repo.git",
            "https://github.com/owner/repo",
        ),
        (
            "https://user:tok@github.com/owner/repo.git/",
            "https://github.com/owner/repo",
        ),
        (
            "http://intranet/owner/repo.git",
            "http://intranet/owner/repo",
        ),
        (
            "git://example.org/pub/repo.git",
            "https://example.org/pub/repo",
        ),
        (
            "git@ssh.dev.azure.com:v3/org/project/repo",
            "https://dev.azure.com/org/project/_git/repo",
        ),
    ] {
        assert_eq!(remote_web_url(url).as_deref(), Some(want), "{url}");
    }
    for not_web in [
        "C:\\repos\\thing",
        "C:/repos/thing",
        "/home/me/repo.git",
        "./relative",
        "file:///srv/git/repo.git",
        "https://github.com/",
        "",
    ] {
        assert_eq!(remote_web_url(not_web), None, "{not_web:?}");
    }
}

#[test]
fn remote_url_validation_accepts_http_and_ssh_shapes_only() {
    for ok in [
        "https://github.com/owner/repo.git",
        "http://host/repo",
        "ssh://git@host:22/owner/repo.git",
        "git@github.com:owner/repo.git",
        "git://example.org/pub/repo.git",
        "file:///srv/git/repo.git",
    ] {
        assert!(validate_remote_url(ok).is_ok(), "{ok}");
    }
    for bad in [
        "",
        "   ",
        "github.com",
        "https://",
        "https://host",
        "C:\\repo",
        "not a url",
        "ftp://h/r",
    ] {
        assert!(validate_remote_url(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn remote_names_follow_git_rules_and_must_be_free() {
    let taken = vec!["origin".to_string(), "fork".to_string()];
    assert!(validate_remote_name("upstream", &taken, None).is_ok());
    assert!(validate_remote_name("grp/sub", &taken, None).is_ok());
    assert!(validate_remote_name("origin", &taken, None).is_err());
    assert!(
        validate_remote_name("origin", &taken, Some("origin")).is_ok(),
        "an edit may keep its own name"
    );
    for bad in ["", "has space", "a..b", "-x", "x/", "x.lock", "a:b", "a~b"] {
        assert!(validate_remote_name(bad, &taken, None).is_err(), "{bad:?}");
    }
}
