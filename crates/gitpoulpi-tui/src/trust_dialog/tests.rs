use super::*;
use crate::dialog_kit::buffer_text;
use crossterm::event::KeyModifiers;
use gitpoulpi_hosting::trust::CertInfo;
use gitpoulpi_hosting::{Account, TrustSubject};

const AREA: Rect = Rect::new(0, 0, 100, 34);

fn origin(name: &str, new: &str) -> TrustProblem {
    TrustProblem::OriginChanged {
        subject: TrustSubject::Integration(name.into()),
        old: "https://old.example.com".into(),
        new: new.into(),
    }
}

fn cert(seed: &str) -> CertInfo {
    CertInfo {
        spki_sha256: seed.repeat(16 / seed.len()).repeat(4)[..64].to_string(),
        subject: "CN=git.example.com".into(),
        issuer: "CN=Example CA".into(),
        not_before: "2026-01-01".into(),
        not_after: "2027-01-01".into(),
    }
}

fn changed_cert(name: &str) -> TrustProblem {
    TrustProblem::CertificateChanged {
        subject: TrustSubject::Integration(name.into()),
        old: Box::new(cert("ab12")),
        new: Box::new(cert("cd34")),
    }
}

fn press(d: &mut Dialog, code: KeyCode) -> Option<Outcome> {
    d.on_key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn screen(d: &mut Dialog, area: Rect) -> (Buffer, String) {
    let mut buf = Buffer::empty(area);
    d.render(&mut buf, area);
    let text = buffer_text(&buf);
    (buf, text)
}

fn click(d: &mut Dialog, at: (u16, u16)) -> Option<Outcome> {
    d.on_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: at.0,
        row: at.1,
        modifiers: KeyModifiers::NONE,
    })
}

fn open_with(problems: Vec<TrustProblem>) -> Dialog {
    let mut d = Dialog::default();
    d.add(problems);
    d
}

#[test]
fn one_question_per_subject_and_kind_the_newest_wins() {
    let mut d = open_with(vec![
        origin("gh", "https://one.example.com"),
        origin("gh", "https://two.example.com"),
        origin("other", "https://three.example.com"),
        changed_cert("gh"),
    ]);
    assert_eq!(d.queue.len(), 3);
    assert_eq!(
        d.current().map(TrustProblem::new_text).as_deref(),
        Some("https://two.example.com")
    );
    let (_, text) = screen(&mut d, AREA);
    assert!(text.contains("More checks are waiting after this one (2)."), "{text}");
}

#[test]
fn the_origin_body_names_the_integration_and_both_addresses() {
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    let (_, text) = screen(&mut d, AREA);
    for want in [
        "Integration security check",
        "work now points to https://new.example.com, it was https://old.example.com.",
        "token was not sent.",
        "1  Disconnect (deletes the saved token)",
        "2  Trust the new address",
        "3  Cancel",
    ] {
        assert!(text.contains(want), "missing {want:?}:\n{text}");
    }
}

#[test]
fn an_account_change_shows_both_logins_and_ids() {
    let mut d = open_with(vec![TrustProblem::AccountChanged {
        subject: TrustSubject::Integration("work".into()),
        old: Box::new(Account {
            login: "alice".into(),
            id: "11".into(),
        }),
        new: Box::new(Account {
            login: "mallory".into(),
            id: "666".into(),
        }),
    }]);
    let (_, text) = screen(&mut d, AREA);
    for want in [
        "Trusted account: alice (id 11)",
        "Account now: mallory (id 666)",
        "2  Trust the new account",
    ] {
        assert!(text.contains(want), "missing {want:?}:\n{text}");
    }
}

#[test]
fn a_certificate_change_is_a_table_with_the_fingerprints_in_groups_of_four() {
    let mut d = open_with(vec![changed_cert("work")]);
    let (_, text) = screen(&mut d, AREA);
    for want in [
        "Trusted before",
        "Presented now",
        "SHA-256",
        "ab12 ab12 ab12",
        "cd34 cd34 cd34",
        "Subject",
        "CN=git.example.com",
        "Issuer",
        "CN=Example CA",
        "Valid from",
        "2026-01-01",
        "Valid to",
        "2027-01-01",
        "2  Trust the new certificate",
    ] {
        assert!(text.contains(want), "missing {want:?}:\n{text}");
    }
}

#[test]
fn a_first_use_shows_the_new_certificate_only() {
    let mut d = open_with(vec![TrustProblem::UntrustedCertificate {
        subject: TrustSubject::Integration("work".into()),
        new: Box::new(cert("ef56")),
    }]);
    let (_, text) = screen(&mut d, AREA);
    assert!(text.contains("no known authority signed"), "{text}");
    assert!(text.contains("Presented now"), "{text}");
    assert!(!text.contains("Trusted before"), "{text}");
}

#[test]
fn a_plain_http_refusal_offers_to_allow_it() {
    let mut d = open_with(vec![TrustProblem::PlainHttp {
        subject: TrustSubject::Integration("work".into()),
        origin: "http://git.example.com".into(),
    }]);
    let (_, text) = screen(&mut d, AREA);
    assert!(text.contains("http://git.example.com, which is not encrypted"), "{text}");
    assert!(text.contains("2  Allow plain http (not encrypted)"), "{text}");
}

#[test]
fn nothing_is_highlighted_until_an_arrow_or_a_digit_and_enter_alone_does_nothing() {
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    assert_eq!(d.cursor, None);
    let selection = crate::th().ui.selection;
    let (buf, _) = screen(&mut d, AREA);
    let highlighted = (0..AREA.height)
        .flat_map(|y| (0..AREA.width).map(move |x| (x, y)))
        .any(|(x, y)| buf[(x, y)].bg == selection);
    assert!(!highlighted, "no answer starts highlighted");
    assert_eq!(press(&mut d, KeyCode::Enter), None);
    assert!(d.is_open(), "Enter with nothing highlighted keeps asking");
    // An arrow lights the first answer, Enter then answers it.
    press(&mut d, KeyCode::Down);
    assert_eq!(d.cursor, Some(0));
    let (buf, _) = screen(&mut d, AREA);
    assert!((0..AREA.height)
        .flat_map(|y| (0..AREA.width).map(move |x| (x, y)))
        .any(|(x, y)| buf[(x, y)].bg == selection));
    let out = press(&mut d, KeyCode::Enter).unwrap();
    assert_eq!(out.choice, Choice::Disconnect);
}

#[test]
fn up_from_nothing_lands_on_the_last_answer() {
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    press(&mut d, KeyCode::Up);
    assert_eq!(d.cursor, Some(2));
}

#[test]
fn digits_answer_at_once() {
    for (digit, choice) in [('1', Choice::Disconnect), ('2', Choice::Trust)] {
        let mut d = open_with(vec![origin("work", "https://new.example.com")]);
        let out = press(&mut d, KeyCode::Char(digit)).unwrap();
        assert_eq!(out.choice, choice);
        assert_eq!(out.problem.subject().name(), "work");
        assert!(!d.is_open());
    }
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    assert_eq!(press(&mut d, KeyCode::Char('3')), None);
    assert!(!d.is_open(), "3 cancels");
    assert_eq!(press(&mut Dialog::default(), KeyCode::Char('1')), None);
}

#[test]
fn esc_cancels_and_a_cancelled_check_does_not_come_back_unchanged() {
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    assert_eq!(press(&mut d, KeyCode::Esc), None);
    assert!(!d.is_open());
    // The same refusal raised again by the next request stays quiet ...
    d.add(vec![origin("work", "https://new.example.com")]);
    assert!(!d.is_open());
    // ... a different one is a new question.
    d.add(vec![origin("work", "https://newer.example.com")]);
    assert!(d.is_open());
    assert_eq!(d.snoozed.len(), 0);
}

#[test]
fn the_review_brings_the_cancelled_checks_back() {
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    press(&mut d, KeyCode::Esc);
    assert!(!d.is_open());
    assert!(d.open_review());
    assert!(d.is_open());
    assert!(!Dialog::default().open_review());
}

#[test]
fn the_answers_go_through_the_queue_one_after_the_other() {
    let mut d = open_with(vec![
        origin("a", "https://a.example.com"),
        origin("b", "https://b.example.com"),
    ]);
    assert_eq!(press(&mut d, KeyCode::Char('3')), None);
    assert_eq!(d.current().unwrap().subject().name(), "b");
    let out = press(&mut d, KeyCode::Char('2')).unwrap();
    assert_eq!(out.problem.subject().name(), "b");
}

#[test]
fn an_ai_profile_cannot_be_disconnected_here() {
    let mut d = open_with(vec![TrustProblem::OriginChanged {
        subject: TrustSubject::Ai("openai".into()),
        old: "https://api.openai.com".into(),
        new: "https://evil.example.com".into(),
    }]);
    assert_eq!(d.choices(), vec![Choice::Trust, Choice::Cancel]);
    let (_, text) = screen(&mut d, AREA);
    assert!(text.contains("1  Trust the new address"), "{text}");
    assert!(text.contains("2  Cancel"), "{text}");
}

#[test]
fn the_cross_cancels_and_a_click_away_is_swallowed() {
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    screen(&mut d, AREA);
    let cross = d.close.rect().expect("the cross is painted");
    // Away from the dialog: nothing happens, it stays.
    assert_eq!(click(&mut d, (0, 0)), None);
    assert!(d.is_open());
    assert_eq!(click(&mut d, (cross.x, cross.y)), None);
    assert!(!d.is_open(), "the cross cancels");
    assert_eq!(d.snoozed.len(), 1);
}

#[test]
fn a_click_on_an_answer_row_answers() {
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    screen(&mut d, AREA);
    let rows = d.painted.answers.clone();
    assert_eq!(rows.len(), 3);
    let out = click(&mut d, (rows[1].x + 3, rows[1].y)).unwrap();
    assert_eq!(out.choice, Choice::Trust);
}

#[test]
fn hovering_an_answer_lights_it_like_the_arrows_do() {
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    screen(&mut d, AREA);
    let row = d.painted.answers[2];
    d.on_mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column: row.x + 2,
        row: row.y,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(d.cursor, Some(2));
}

/// A dialog too short for its table: the body scrolls, with a bar that drags.
#[test]
fn a_long_body_scrolls_with_keys_the_wheel_and_a_draggable_bar() {
    let small = Rect::new(0, 0, 100, 18);
    let mut d = open_with(vec![changed_cert("work")]);
    let (_, text) = screen(&mut d, small);
    assert!(text.contains("Trusted before"), "{text}");
    assert!(!text.contains("Valid to  2027"), "the end is below the fold:\n{text}");
    let track = d.painted.track.expect("a bar is drawn when the body is cut");
    // Keys.
    press(&mut d, KeyCode::PageDown);
    assert!(d.scroll > 0);
    press(&mut d, KeyCode::Home);
    assert_eq!(d.scroll, 0);
    press(&mut d, KeyCode::End);
    let end = d.scroll;
    assert!(end > 0);
    let (_, text) = screen(&mut d, small);
    assert!(text.contains("2027-01-01"), "the end is shown at the end:\n{text}");
    // Wheel.
    press(&mut d, KeyCode::Home);
    let inside = (d.painted.window.x + 5, d.painted.window.y + 3);
    d.on_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: inside.0,
        row: inside.1,
        modifiers: KeyModifiers::NONE,
    });
    assert!(d.scroll > 0);
    // The thumb drags: press on it, move to the bottom of the track.
    press(&mut d, KeyCode::Home);
    screen(&mut d, small);
    let track = d.painted.track.unwrap_or(track);
    let thumb_row = track.rect.y + track.thumb_start;
    click(&mut d, (track.rect.x, thumb_row));
    d.on_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: track.rect.x,
        row: track.rect.bottom() - 1,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(d.scroll, end, "dragging the thumb to the end scrolls to the end");
    d.on_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: track.rect.x,
        row: track.rect.bottom() - 1,
        modifiers: KeyModifiers::NONE,
    });
    assert!(d.is_open());
}

#[test]
fn a_short_body_draws_no_bar() {
    let mut d = open_with(vec![origin("work", "https://new.example.com")]);
    screen(&mut d, AREA);
    assert!(d.painted.track.is_none());
}

// -- apply: the answers' effects over a private directory ---------------------

mod effects {
    use super::*;
    use gitpoulpi_config::{set_list_in_file, ConfigPaths, Integration, IntegrationKind};
    use gitpoulpi_hosting::TrustGate;

    fn setup(url: &str) -> (tempfile::TempDir, ConfigPaths, Integration) {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        let integration = Integration::new("work", IntegrationKind::GitlabSelfManaged, url);
        set_list_in_file(&paths.config_file(), "integrations", &[integration.clone()]).unwrap();
        (dir, paths, integration)
    }

    fn answer(paths: &ConfigPaths, problem: TrustProblem, choice: Choice) -> Option<String> {
        let mut message = None;
        let mut preferences = None;
        super::super::apply::apply_in(
            paths,
            Outcome { problem, choice },
            std::path::Path::new("."),
            &mut preferences,
            &mut message,
        );
        assert!(preferences.is_none() || choice == Choice::Trust);
        message
    }

    #[test]
    fn trusting_an_account_records_it() {
        let (_dir, paths, integration) = setup("https://git.example.com");
        let message = answer(
            &paths,
            TrustProblem::AccountChanged {
                subject: TrustSubject::Integration("work".into()),
                old: Box::default(),
                new: Box::new(Account {
                    login: "new".into(),
                    id: "42".into(),
                }),
            },
            Choice::Trust,
        );
        assert_eq!(message.as_deref(), Some("work: the new account is trusted."));
        let record = TrustGate::load(&paths).record_of("work").unwrap();
        assert_eq!((record.account_id.as_str(), record.account_login.as_str()), ("42", "new"));
        let _ = integration;
    }

    #[test]
    fn trusting_a_certificate_pins_it_and_trusting_plain_http_allows_it() {
        let (_dir, paths, _) = setup("http://git.example.com");
        answer(
            &paths,
            TrustProblem::UntrustedCertificate {
                subject: TrustSubject::Integration("work".into()),
                new: Box::new(cert("ef56")),
            },
            Choice::Trust,
        );
        let record = TrustGate::load(&paths).record_of("work").unwrap();
        assert_eq!(record.tls.as_ref().unwrap().cert.spki_sha256, cert("ef56").spki_sha256);
        assert_eq!(record.tls.unwrap().mode, gitpoulpi_hosting::trust::TlsMode::Tofu);
        let message = answer(
            &paths,
            TrustProblem::PlainHttp {
                subject: TrustSubject::Integration("work".into()),
                origin: "http://git.example.com".into(),
            },
            Choice::Trust,
        );
        assert_eq!(message.as_deref(), Some("work: plain http is allowed."));
        assert!(TrustGate::load(&paths).record_of("work").unwrap().allow_plain_http);
    }

    #[test]
    fn an_address_that_changed_again_is_not_trusted() {
        let (_dir, paths, _) = setup("https://third.example.com");
        let message = answer(
            &paths,
            origin("work", "https://second.example.com"),
            Choice::Trust,
        );
        assert_eq!(
            message.as_deref(),
            Some("work changed again: nothing was trusted. Review it again.")
        );
        assert!(TrustGate::load(&paths).record_of("work").is_none());
    }

    #[test]
    fn an_integration_that_is_gone_says_so() {
        let (_dir, paths, _) = setup("https://git.example.com");
        let message = answer(&paths, origin("missing", "https://x.example.com"), Choice::Trust);
        assert_eq!(
            message.as_deref(),
            Some("missing is no longer in the configuration.")
        );
    }

    #[test]
    fn trusting_an_ai_profile_origin_records_it() {
        let (_dir, paths, _) = setup("https://git.example.com");
        let message = answer(
            &paths,
            TrustProblem::OriginChanged {
                subject: TrustSubject::Ai("openai".into()),
                old: "https://api.openai.com".into(),
                new: "https://proxy.example.com".into(),
            },
            Choice::Trust,
        );
        assert_eq!(message.as_deref(), Some("openai: the new address is trusted."));
        let record = TrustGate::load(&paths).ai_record_of("openai").unwrap();
        assert_eq!(record.origin, "https://proxy.example.com");
    }
}
