use cargo_registry::app::App;
use cargo_registry::badge::Badge;
use cargo_registry::krate::Crate;

use std::collections::HashMap;
use std::sync::Arc;

struct BadgeRef {
    appveyor: Badge,
    appveyor_attributes: HashMap<String, String>,
    travis_ci: Badge,
    travis_ci_attributes: HashMap<String, String>,
    gitlab: Badge,
    gitlab_attributes: HashMap<String, String>,
}

fn set_up() -> (Arc<App>, Crate, BadgeRef) {
    let (_b, app, _middle) = ::app();

    let krate = {
        let conn = app.diesel_database.get().unwrap();
        let u = ::new_user("foo")
            .create_or_update(&conn)
            .unwrap();
        ::new_crate("badged_crate")
            .create_or_update(&conn, None, u.id)
            .unwrap()
    };

    let appveyor = Badge::Appveyor {
        service: Some(String::from("github")),
        branch: None,
        repository: String::from("rust-lang/cargo"),
    };
    let mut badge_attributes_appveyor = HashMap::new();
    badge_attributes_appveyor.insert(
        String::from("service"),
        String::from("github")
    );
    badge_attributes_appveyor.insert(
        String::from("repository"),
        String::from("rust-lang/cargo")
    );

    let travis_ci = Badge::TravisCi {
        branch: Some(String::from("beta")),
        repository: String::from("rust-lang/rust"),
    };
    let mut badge_attributes_travis_ci = HashMap::new();
    badge_attributes_travis_ci.insert(
        String::from("branch"),
        String::from("beta")
    );
    badge_attributes_travis_ci.insert(
        String::from("repository"),
        String::from("rust-lang/rust")
    );

    let gitlab = Badge::GitLab {
        branch: Some(String::from("beta")),
        repository: String::from("rust-lang/rust"),
    };
    let mut badge_attributes_gitlab = HashMap::new();
    badge_attributes_gitlab.insert(
        String::from("branch"),
        String::from("beta")
    );
    badge_attributes_gitlab.insert(
        String::from("repository"),
        String::from("rust-lang/rust")
    );

    let badges = BadgeRef {
        appveyor: appveyor,
        appveyor_attributes: badge_attributes_appveyor,
        travis_ci: travis_ci,
        travis_ci_attributes: badge_attributes_travis_ci,
        gitlab: gitlab,
        gitlab_attributes: badge_attributes_gitlab,
    };
    (app, krate, badges)
}

#[test]
fn update_no_badges() {
    // Add no badges
    let (app, krate, _) = set_up();
    let conn = app.diesel_database.get().unwrap();

    // Updating with no badges has no effect
    Badge::update_crate(&conn, &krate, None).unwrap();
    assert_eq!(krate.badges(&conn).unwrap(), vec![]);
}

#[test]
fn update_add_appveyor() {
    // Add an appveyor badge
    let (app, krate, test_badges) = set_up();
    let conn = app.diesel_database.get().unwrap();

    let mut badges = HashMap::new();
    badges.insert(
        String::from("appveyor"),
        test_badges.appveyor_attributes
    );
    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(krate.badges(&conn).unwrap(), vec![test_badges.appveyor]);
}

#[test]
fn update_add_travis_ci() {
    // Add a travis ci badge
    let (app, krate, test_badges) = set_up();
    let conn = app.diesel_database.get().unwrap();

    let mut badges = HashMap::new();
    badges.insert(
        String::from("travis-ci"),
        test_badges.travis_ci_attributes
    );
    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(krate.badges(&conn).unwrap(), vec![test_badges.travis_ci]);
}

#[test]
fn update_add_gitlab() {
    // Add a gitlab badge
    let (app, krate, test_badges) = set_up();
    let conn = app.diesel_database.get().unwrap();

    let mut badges = HashMap::new();
    badges.insert(
        String::from("gitlab"),
        test_badges.gitlab_attributes
    );
    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(krate.badges(&conn).unwrap(), vec![test_badges.gitlab]);
}

#[test]
fn replace_badge() {
    // Replacing one badge with another
    let (app, krate, test_badges) = set_up();
    let conn = app.diesel_database.get().unwrap();

    // Add a badge
    let mut badges = HashMap::new();
    badges.insert(
        String::from("gitlab"),
        test_badges.gitlab_attributes
    );
    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(krate.badges(&conn).unwrap(), vec![test_badges.gitlab]);

    // Replace with another badge
    badges.clear();
    badges.insert(
        String::from("travis-ci"),
        test_badges.travis_ci_attributes.clone()
    );
    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(krate.badges(&conn).unwrap(), vec![test_badges.travis_ci]);
}

#[test]
fn update_attributes() {
    // Update badge attributes
    let (app, krate, test_badges) = set_up();
    let conn = app.diesel_database.get().unwrap();

    // Add a travis-ci badge
    let mut badges = HashMap::new();
    badges.insert(
        String::from("travis-ci"),
        test_badges.travis_ci_attributes
    );
    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    let current_badges = krate.badges(&conn).unwrap();
    assert_eq!(current_badges.len(), 1);
    assert!(current_badges.contains(&test_badges.travis_ci));

    // Now update the travis ci badge with different attributes
    let mut badges = HashMap::new();
    let travis_ci2 = Badge::TravisCi {
        branch: None,
        repository: String::from("rust-lang/rust"),
    };
    let mut badge_attributes_travis_ci2 = HashMap::new();
    badge_attributes_travis_ci2.insert(
        String::from("repository"),
        String::from("rust-lang/rust")
    );
    badges.insert(
        String::from("travis-ci"),
        badge_attributes_travis_ci2.clone()
    );
    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    let current_badges = krate.badges(&conn).unwrap();
    assert_eq!(current_badges.len(), 1);
    assert!(current_badges.contains(&travis_ci2));
}

#[test]
fn clear_badges() {
    // Add 3 badges and then remove them
    let (app, krate, test_badges) = set_up();
    let conn = app.diesel_database.get().unwrap();

    let mut badges = HashMap::new();

    // Adding 3 badges
    badges.insert(
        String::from("appveyor"),
        test_badges.appveyor_attributes
    );
    badges.insert(
        String::from("travis-ci"),
        test_badges.travis_ci_attributes
    );
    badges.insert(
        String::from("gitlab"),
        test_badges.gitlab_attributes
    );
    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();

    let current_badges = krate.badges(&conn).unwrap();
    assert_eq!(current_badges.len(), 3);
    assert!(current_badges.contains(&test_badges.appveyor));
    assert!(current_badges.contains(&test_badges.travis_ci));
    assert!(current_badges.contains(&test_badges.gitlab));

    // Removing all badges
    badges.clear();
    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(krate.badges(&conn).unwrap(), vec![]);
}

#[test]
fn appveyor_extra_keys() {
    // Add a badge with extra invalid keys
    let (app, krate, test_badges) = set_up();
    let conn = app.diesel_database.get().unwrap();

    let mut badges = HashMap::new();

    // Extra invalid keys are fine, they just get ignored
    let mut appveyor_attributes = test_badges.appveyor_attributes.clone();
    appveyor_attributes.insert(
        String::from("extra"),
        String::from("info")
    );
    badges.insert(
        String::from("appveyor"),
        test_badges.appveyor_attributes
    );

    Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(krate.badges(&conn).unwrap(), vec![test_badges.appveyor]);
}

#[test]
fn travis_ci_required_keys() {
    // Add a travis ci badge missing a required field
    let (app, krate, mut test_badges) = set_up();
    let conn = app.diesel_database.get().unwrap();

    let mut badges = HashMap::new();

    // Repository is a required key
    test_badges.travis_ci_attributes.remove("repository");
    badges.insert(
        String::from("travis-ci"),
        test_badges.travis_ci_attributes
    );

    let invalid_badges = Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(invalid_badges.len(), 1);
    assert!(invalid_badges.contains(&"travis-ci"));
    assert_eq!(krate.badges(&conn).unwrap(), vec![]);
}

#[test]
fn gitlab_required_keys() {
    // Add a gitlab badge missing a required field
    let (app, krate, mut test_badges) = set_up();
    let conn = app.diesel_database.get().unwrap();

    let mut badges = HashMap::new();

    // Repository is a required key
    test_badges.gitlab_attributes.remove("repository");
    badges.insert(
        String::from("gitlab"),
        test_badges.gitlab_attributes
    );

    let invalid_badges = Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(invalid_badges.len(), 1);
    assert!(invalid_badges.contains(&"gitlab"));
    assert_eq!(krate.badges(&conn).unwrap(), vec![]);
}

#[test]
fn unknown_badge() {
    // Add an unknown badge
    let (app, krate, _) = set_up();
    let conn = app.diesel_database.get().unwrap();

    let mut badges = HashMap::new();

    // This is not a badge that crates.io knows about
    let mut invalid_attributes = HashMap::new();
    invalid_attributes.insert(
        String::from("not-a-badge-attribute"),
        String::from("not-a-badge-value")
    );
    badges.insert(
        String::from("not-a-badge"),
        invalid_attributes
    );

    let invalid_badges = Badge::update_crate(&conn, &krate, Some(&badges)).unwrap();
    assert_eq!(invalid_badges.len(), 1);
    assert!(invalid_badges.contains(&"not-a-badge"));
    assert_eq!(krate.badges(&conn).unwrap(), vec![]);
}
