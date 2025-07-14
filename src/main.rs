use mankeli_chat::db::{User, retr_user, setup_db};
use rusqlite::{Connection, Result, params};
use std::io::{self, Write};

// TO-DO write a cli menu next

#[cfg(test)]
mod tests;

fn main() {
    let conn = Connection::open_in_memory().unwrap();

    let username = read_input("Enter Username: ");

    //TO-DO fetch if exists in db if exist greet if not then promt and ask if you want to create new account

    println!("\nWelcome {}!\n", &username);

    let user: User = retr_user(&conn).unwrap_or_else(|_| init_db(&conn, username));

    loop {
        let prompt =
            "\nAvailable commands: inbox, friends, send, outbound, quit\nPlease enter something: ";

        let cmd = read_input(prompt).to_lowercase();

        match cmd.as_str() {
            "inbox" => read_inbox(&conn),
            "friends" => read_friends(&conn),
            "send" => send_message(&conn),
            "outbound" => view_outbound(&conn),
            "quit" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Unknown command."),
        }
    }
}

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

fn init_db(conn: &Connection, username: String) -> User {
    let user = User {
        id: 0,
        username: username,
        address: "127.0.0.1".to_string(),
    };

    let res: std::result::Result<(), rusqlite::Error> = setup_db(&conn, &user);

    user
}

fn read_inbox(conn: &Connection) {
    println!("you dont have mail")
}

fn read_friends(conn: &Connection) {
    todo!()
}

fn send_message(conn: &Connection) {
    //who?
    //message body?

    todo!()
}

fn view_outbound(conn: &Connection) {
    todo!()
}
