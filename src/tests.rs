use super::*;

#[test]
fn test_user_init() {
    let conn = Connection::open_in_memory().unwrap();

    let uname: String = "foobar".to_string();

    let user: User = retr_user(&conn).unwrap_or_else(|_| init_db(&conn, uname));

    assert_eq!(user.address, "127.0.0.1");
    assert_eq!(user.username, "foobar");
}
