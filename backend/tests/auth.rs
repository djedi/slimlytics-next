use slimlytics_backend::auth::{hash_password, issue_token, verify_password, verify_token};
use uuid::Uuid;

#[test]
fn password_hashes_are_argon2_and_verify() {
    let hash = hash_password("correct horse battery staple").unwrap();
    assert!(hash.starts_with("$argon2"));
    assert!(verify_password("correct horse battery staple", &hash).unwrap());
    assert!(!verify_password("wrong", &hash).unwrap());
}

#[test]
fn jwt_round_trip_rejects_wrong_secret() {
    let id = Uuid::new_v4();
    let token = issue_token(id, "secret", 60).unwrap();
    let claims = verify_token(&token, "secret").unwrap();
    assert_eq!(claims.sub, id);
    assert_eq!(claims.exp - claims.iat, 60);
    assert!(verify_token(&token, "other").is_err());
}
