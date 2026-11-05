use alexia::auth::Sessions;

#[test]
fn bootstrap_is_single_use_expiring_and_rate_limited() {
    let (mut sessions, code) = Sessions::new(100);
    assert!(sessions.bootstrap("wrong", 101).is_err());
    let session = sessions.bootstrap(&code, 102).unwrap();
    assert!(sessions.bootstrap(&code, 103).is_err());
    assert!(
        sessions
            .authenticate(&session.token, &session.csrf, 103)
            .is_ok()
    );
    assert_eq!(
        sessions.resume(&session.token, 103).unwrap().1,
        session.csrf
    );
    assert_eq!(
        sessions.resume(&session.token, 104).unwrap().1,
        session.csrf
    );
    assert!(
        sessions
            .authenticate(&session.token, &session.csrf, 105)
            .is_ok()
    );
    assert!(sessions.authenticate(&session.token, "wrong", 103).is_err());
    assert!(
        sessions
            .authenticate(&session.token, &session.csrf, 40_000)
            .is_err()
    );
    let (mut expired, code) = Sessions::new(100);
    assert!(expired.bootstrap(&code, 701).is_err());
    let (mut limited, code) = Sessions::new(100);
    for _ in 0..10 {
        assert!(limited.bootstrap("wrong", 101).is_err());
    }
    assert!(limited.bootstrap(&code, 102).is_err());
}

#[test]
fn paired_client_has_distinct_identity_and_revocation() {
    let (mut sessions, code) = Sessions::new(100);
    let owner = sessions.bootstrap(&code, 101).unwrap();
    let pairing = sessions
        .pairing_code(&owner.token, &owner.csrf, "editor", 102)
        .unwrap();
    assert!(sessions.pair(&pairing, "other", 103).is_err());
    let client = sessions.pair(&pairing, "editor", 103).unwrap();
    assert_ne!(client.principal_id, owner.principal_id);
    assert!(sessions.pair(&pairing, "editor", 104).is_err());
    sessions.revoke(&client.principal_id);
    assert!(
        sessions
            .authenticate(&client.token, &client.csrf, 105)
            .is_err()
    );
}
