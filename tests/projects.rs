use alexia::projects::Project;
use std::fs;

#[test]
fn remote_projects_need_identity_but_no_checkout() {
    let project = Project::remote("owner", "connection", 42);
    assert_eq!(project.owner_id, "owner");
    assert!(project.read_context("../.env", 1024).is_err());
}

#[test]
fn local_context_rejects_traversal_secrets_and_other_roots() {
    let root = std::env::temp_dir().join(format!("alexia-project-{}", std::process::id()));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/mañana.rs"), "fn main() {}").unwrap();
    fs::write(root.join(".env"), "SYNTHETIC=private").unwrap();
    let project = Project::local("owner", &root).unwrap();
    assert_eq!(
        project.read_context("src/mañana.rs", 100).unwrap(),
        "fn main() {}"
    );
    for path in ["../outside", ".env", "src/../.env", "C:\\Windows\\win.ini"] {
        assert!(project.read_context(path, 100).is_err(), "{path}");
    }
    assert!(project.read_context("src/mañana.rs", 3).is_err());
}
