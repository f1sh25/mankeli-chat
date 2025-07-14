use mankeli_chat::db::{
    FriendRequest, User, delete_message, delete_user, fetch_inbox, fetch_users, retr_user,
    send_invite, setup_db,
};
use rusqlite::Connection;
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
    let inbox = match fetch_inbox(conn) {
        Ok(messages) => messages,
        Err(e) => {
            eprintln!("Error fetching inbox: {}", e);
            return;
        }
    };

    if inbox.is_empty() {
        println!("You don't have any mail.");
        return;
    }

    println!("Your inbox:");
    for (i, message) in inbox.iter().enumerate() {
        println!(
            "{}. From: {}, Subject: {}",
            i + 1,
            message.sender,
            message.subject
        );
    }

    loop {
        print!("\nEnter message number to read in full, or 'b' to go back: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if let Err(_) = io::stdin().read_line(&mut input) {
            println!("Failed to read input. Try again.");
            continue;
        }

        let input = input.trim();

        if input.eq_ignore_ascii_case("b") {
            println!("Returning to main menu...");
            break;
        }

        match input.parse::<usize>() {
            Ok(index) if index > 0 && index <= inbox.len() => {
                let message = &inbox[index - 1];
                println!(
                    "\nFrom: {}\nSubject: {}\n\n{}",
                    message.sender, message.subject, message.message
                );

                print!("Delete this message? (y/n): ");
                io::stdout().flush().unwrap();

                let mut del_input = String::new();
                if let Ok(_) = io::stdin().read_line(&mut del_input) {
                    if del_input.trim().eq_ignore_ascii_case("y") {
                        match delete_message(conn, message.id) {
                            Ok(_) => println!("Message deleted."),
                            Err(e) => println!("Failed to delete message: {}", e),
                        }
                    }
                }

                break;
            }
            _ => {
                println!("Invalid input. Please enter a valid number or 'b'.");
            }
        }
    }
}

fn read_friends(conn: &Connection) {
    let friends = match fetch_users(conn) {
        Ok(friends) => friends,
        Err(e) => {
            eprintln!("Error fetching users: {}", e);
            return;
        }
    };

    println!("your friends: ");
    for fr in friends {
        println!("{}. {}", fr.id, fr.username,)
    }

    loop {
        let response = read_input("a: Add Friend, r: remove Friend, b: go back: ").to_lowercase();

        if response.as_str() == "b" {
            println!("Returning to main menu...");
            break;
        }

        match response.as_str() {
            "a" => {
                let username = read_input("Enter username of user: ");
                let address = read_input("Enter ip/hostname of user: ");
                let _ = send_invite(
                    conn,
                    FriendRequest {
                        username: username,
                        address: address,
                    },
                )
                .unwrap();
            }
            "r" => {
                let id = read_input("enter friend id to remove them: ");

                match id.trim().parse::<i32>() {
                    Ok(friend_id) => {
                        println!("Removing friend with id: {}", friend_id);
                        let _ = delete_user(conn, friend_id);
                    }
                    Err(e) => {
                        println!("Invalid input: must be a number. Error: {}", e);
                    }
                }
            }
            _ => {
                println!("Invalid input. Please enter a valid number or 'b'.");
            }
        }
    }
}

fn send_message(conn: &Connection) {
    //who?
    //message body?

    todo!()
}

fn view_outbound(conn: &Connection) {
    todo!()
}
