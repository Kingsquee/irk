#![allow(unused_variables)]
#![allow(dead_code)]

extern crate irc;

use std::default::Default;
use irc::client::prelude::*;
use irc::client::data::user::User;
use std::collections::HashMap;

const BOT_NAME: &'static str = "irken";
const STARTING_COIN_NAME: &'static str = "shit coin";
const STARTING_COIN_COUNT: u64 = 100;

#[derive(PartialEq)]
enum Subject {
    Speaker,
    Other,
    Bot,
}
/*
struct Room {
    description: String,
    characters: Vec<String>,
    items: Vec<ItemStack>,
}
*/

#[derive(PartialEq, Debug)]
struct Character {
    name: String,
    description: String,
    inventory: HashMap<String, u64>,
}

#[derive(PartialEq, Clone, Debug)]
struct Item {
    name: String,
    description: String,
    creator_name: String
}

macro_rules! parse_message_for_commands {
    ($server:expr, $message:expr, $items:expr, $characters:expr, $( $name:ident ),+ ) => {
        {
            if $message.get_source_nickname() == None { continue }
            let source_nickname = $message.get_source_nickname().unwrap();
            //println!("source_nickname: {:?}", source_nickname);

            if $message.command[..] != "PRIVMSG".to_string() { continue }

            if $message.suffix == None { continue }
            let msg = $message.suffix.clone().unwrap();
            //println!("{}", msg);


            let mut trimmed_msg;
            if msg.starts_with("!") {
                trimmed_msg = msg.trim_left_matches("!");
            } else if msg.starts_with("irk:") {
                trimmed_msg = msg.trim_left_matches("irk:");
            } else if msg.starts_with("irk,") {
                trimmed_msg = msg.trim_left_matches("irk,");
            } else {
                continue
            }
            $(
                let mut args: Vec<&str> = trimmed_msg.split_whitespace().collect();
                //println!("args: {:?}", args);

                //print!("Does {} start with {}? ", args[0], stringify!($name));
                if args[0] == stringify!($name) {
                    args.remove(0);
                    //args.map(|s| s.trim_matches(r"'s"));
                    remove_ownership_suffixes(&mut args);
                    let response = $name(&source_nickname, &args, &mut $items, &mut $characters);
                    $server.send_privmsg(&$message.args[0], &response).unwrap_or_else(|e| {
                        println!("ERROR: Could not send response: {}", e);
                    });
                    continue
                } else {
                    //println!("No.");
                }
            )+
        }
    };
}

fn main() {

    let channel_name = format!("#{}", "boatdev");

    let config = Config {
        nickname: Some(BOT_NAME.to_string()),
        server: Some(format!("chat.freenode.net")),
        channels: Some(vec![channel_name.clone()]),
        port: Some(7070),
        use_ssl: Some(true),
        .. Default::default()
    };

    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();
/*
    let mut room = Room {
        description: "A small, dank room surrounds you.".to_string(),
        characters: HashMap::new(),
        items: HashMap::new(),
    };
*/

    let mut items:      HashMap<String, Item> = HashMap::new();

    items.insert(STARTING_COIN_NAME.to_string(), Item {
        name: STARTING_COIN_NAME.to_string(),
        description: "finely crafted, of the highest quality".to_string(),
        creator_name: "GOD ALMIGHTY".to_string()
    });

    let mut characters: HashMap<String, Character> = HashMap::new();

    if let Some(users) = server.list_users(&channel_name) {
        update_characters(&users, &mut characters);
    }  else {
        println!("ERROR: Could not get user list");
            print!("Characters are:");
            println!("{:?}", characters);
    }


    for message in server.iter() {
        if let Some(users) = server.list_users(&channel_name) {
            update_characters(&users, &mut characters);
            print!("Characters are:");
            println!("{:?}", characters);
        } else {
            println!("ERROR: Could not get user list");
        }
        let message = message.unwrap();
        print!("{}", message.into_string());
        parse_message_for_commands!(server, message, items, characters,
        hi,
        I,  // describe yourself
        a,  // describe a <item>
        an, // describe an <item>
        make,
        create,
        craft,
        look,
        equip,
        give,
        bid,
        quit
        );
    }
}

fn remove_ownership_suffixes(tokens: &mut Vec<&str>) {
    for token in tokens.iter_mut() {
        *token = token.trim_right_matches("\'s");
    }
}

fn update_characters(users: &Vec<User>, characters: &mut HashMap<String, Character>) {

    let mut names_to_remove: Vec<String> = Vec::with_capacity(characters.len());
    'char_root_iter: for (name, _) in characters.iter_mut() {
        for user in users.iter() {
            if name == user.get_name() {
                continue 'char_root_iter
            }
        }
        names_to_remove.push(name.clone());
    }

    for name in names_to_remove.iter() {
        characters.remove(name);
    }

    'user_root_iter: for user in users.iter() {
        let username = user.get_name();
        if characters.contains_key(username) || username == "" {
            continue
        }
        // if we hit here, the username isn't in the character list
        // add character
        characters.insert(username.to_string(), Character {
            name: username.to_string(),
            description: "an average shitposter".to_string(),
            inventory: HashMap::new(),
        });
        characters.get_mut(username).unwrap().inventory.insert(STARTING_COIN_NAME.to_string(), 100u64);
    }
}

fn hi(source_nickname: &str, args: &Vec<&str>, items: &HashMap<String, Item>, characters: &HashMap<String, Character>) -> String {
    format!("Hi {}!", source_nickname)
}

//TODO:THINK: Remove 'me' 'myself', etc before passing to parser functions?

/// Syntax:
/// look at me
/// look at user
/// look at item
/// look at my item
/// look at user's item

fn look(source_nickname: &str, args: &Vec<&str>, items: &HashMap<String, Item>, characters: &HashMap<String, Character>) -> String {

    fn look_at_character(character: &Character, subject_type: Subject) -> String {
            format!("{segway}{description}. {inventory}",
                segway =    match subject_type {
                                Subject::Speaker => { format!("{}, you are ", character.name) }
                                Subject::Other => { format!("{} is ", character.name) }
                                Subject::Bot => { format!("I am ") }
                            },
                description = character.description,
                inventory = if character.inventory.len() > 0 {
                    let mut s = match subject_type {
                        Subject::Speaker => { format!("You have on your person: ") }
                        Subject::Other => { format!("They have on their person: ") }
                        Subject::Bot => { format!("I have on my person: ") }
                    };

                    println!("{:?}", character.inventory);
                    let mut index = 0;
                    for (name, count) in character.inventory.iter() {
                        s.push_str(&format!("{and}{count} {name}{comma}",
                        and = if character.inventory.len() > 1 && index == character.inventory.len() - 1 { "and " } else {""},
                        count = if *count == 1 {
                                    if name.starts_with("a") {
                                        "an".to_string()
                                    } else {
                                        "a".to_string()
                                    }
                                } else {
                                    count.to_string()
                                },
                        name = name,
                        comma = if index == character.inventory.len() - 1 { "" } else { ", " }));

                        index += 1;
                    }
                    s

                } else {
                    match subject_type {
                        Subject::Speaker => { format!("You have nothing to your name!") }
                        Subject::Other => { format!("They have nothing to their name!") }
                        Subject::Bot => { format!("I have nothing to my name!") }
                    }
                }
            )
        }

    // My <item name> is <desc>
    // My <item name> are <desc>
    // Your <item name> is <desc>
    // Your <item name> are <desc>
    // Their <item name> is <desc>
    // Their <item name> are <desc>
    fn look_at_inventory_item(item: &Item, item_count: u64, subject_type: Subject) -> String {
        format!("{ownership} {name} {segway} {description}. It was crafted with love by {creator}.",
            ownership = match subject_type {
                Subject::Speaker => { "Your" }
                Subject::Other => { "Their" }
                Subject::Bot => { "My" }
            },
            name = item.name,
            segway = if item_count == 1 { "is" } else { "are" },
            description = item.description,
            creator = item.creator_name,
        )
    }

    if args.len() == 0 {

        return format!("I see {names}",
            names = if characters.len() > 0 {
                let mut s = String::new();
                let mut index = 0;
                for (name, _) in characters.iter() {
                    s.push_str(&format!("{name}{comma}",
                        name = name,
                        comma = if index == characters.len() - 1 { "" } else { ", " }
                    ));
                    index += 1;
                }
                s
            } else {
                format!("nobody!")
            }
        )
    }
    // TODO: Parse character description for gender (if "she" is found and "he" isn't then female, reversed, male, if both found, androgynous)

    let noun_start_index = if args[0] == "at" {
        1
    } else {
        0
    };

    if args.get(noun_start_index).is_none() {
        return format!("Look at what, {}?", source_nickname)
    }

    // / look at a
    // x look at b
    // / look at a's b

    if args.get(noun_start_index + 1).is_some() {
        // Talking about thing's ownership
        println!("Talking about an owned item");

        let item_name = get_item_name(&args[noun_start_index + 1..args.len()]);
        let character_name = if args[noun_start_index] == "my" { source_nickname } else { args[noun_start_index] };

        let character;
        if let Some(c) = characters.get(character_name) {
            character = c;
        } else {
            return format!("Who?");
        }
        let item;
        if let Some(i) = items.get(&item_name) {
            item = i;
        } else if let Some(i) = items.get(item_name.trim_right_matches("s")) {
            item = i;
        } else {
            return format!("Sorry, what item?");
        }

        let item_count;
        if let Some(c) = character.inventory.get(&item_name) {
            item_count = c;
        } else if let Some(c) = character.inventory.get(item_name.trim_right_matches("s")) {
            item_count = c;
        } else {
            return format!("Sorry, get what now?")
        }

        let subject = match args[noun_start_index] {
            "my"        => Subject::Speaker,
            BOT_NAME    => Subject::Bot,
            _           => Subject::Other
        };
        return look_at_inventory_item(&item, *item_count, subject)
    } else {
        // Talking about character
        println!("Talking about a character: {}", args[noun_start_index]);
            if args[noun_start_index] == "me" || args[noun_start_index] == "myself" || args[noun_start_index] == source_nickname {
                return match characters.get(source_nickname) {
                    Some(character) => { look_at_character(character, Subject::Speaker) }
                    None => { format!("Who the hell are you?") }
                }
            } else if args[noun_start_index] == BOT_NAME {
                return match characters.get(args[noun_start_index]) {
                    Some(character) => { look_at_character(character, Subject::Bot) }
                    None => { format!("Do I know myself? Who am I, really!") }
                }
            } else {
                return match characters.get(args[noun_start_index]) {
                    Some(character) => { look_at_character(character, Subject::Other) }
                    None => { format!("I don't know this strange person?") }
                }
            }
    }
    return format!("W-what?");
}

/// Creates an item with the specified name
// create a thing
fn create(source_nickname: &str, args: &Vec<&str>, items: &mut HashMap<String, Item>, characters: &mut HashMap<String, Character>) -> String {
    make(source_nickname, args, items, characters)
}
fn craft(source_nickname: &str, args: &Vec<&str>, items: &mut HashMap<String, Item>, characters: &mut HashMap<String, Character>) -> String {
    make(source_nickname, args, items, characters)
}
fn make(source_nickname: &str, args: &Vec<&str>, items: &mut HashMap<String, Item>, characters: &mut HashMap<String, Character>) -> String {

    if args.len() <= 1 {
        return format!("ESPANOL ES NO MIO.")
    }

    if args[0].trim().parse::<isize>().is_ok() {
        return format!("You can't make more than one item at a time, dweeb.")
    }

    let noun_start_index = if args[0] == "a" || args[0] == "an" || args[0] == "some" {
        1
    } else {
        0
    };

    let item_name = get_item_name(&args[noun_start_index..args.len()]);

    // Detect counterfieting
    if item_name == STARTING_COIN_NAME {
        return format!("You can't create a {}! That's COUNTERFIETING!.", STARTING_COIN_NAME)
    }

    // Detect copyright infringement
    if let Some(existing_item) = items.get(&item_name) {
        if existing_item.creator_name != source_nickname {
            return format!("You can't create a {}! That's COPYRIGHT INFRINGEMENT on {}!", item_name, existing_item.creator_name)
        }
    }

    let character;
    if let Some(c) = characters.get_mut(source_nickname) {
        character = c;
    } else {
        return format!("Who ARE you?!")
    }

    let item = Item {
        name: item_name.clone(),
        description: "mysterious".to_string(),
        creator_name: source_nickname.to_string()
    };
    items.insert(item_name.clone(), item);
    if character.inventory.contains_key(&item_name) {
        *character.inventory.get_mut(&item_name).unwrap() += 1;
    } else {
        character.inventory.insert(item_name, 1u64);
    }
    return format!("A sparkle of light, and it is done!")
}

fn get_item_name(args: &[&str]) -> String {
    let mut item_name: String = String::new();
    for i in 0..args.len() {
        item_name.push_str(&format!("{}{}", args[i], if i == args.len()-1 { "" } else { " " }));
    }
    println!("GOT ITEM NAME: {}", item_name);
    item_name
}

/// Adds a description to you, or the specified item you're carrying IF you made it.
/// Syntax:
/// !I am <desc>
#[allow(non_snake_case)]
fn I(source_nickname: &str, args: &Vec<&str>, items: &HashMap<String, Item>, characters: &mut HashMap<String, Character>) -> String {

    if !characters.contains_key(source_nickname) {
        return format!("YOU. ARE. NOT. WORTHY. ALSO THIS IS A BUG.")
    }

    if args.len() == 0 {
        return format!("YOU.")
    } else if args.len() == 1 {
        return format!("YOU IS WHAT.")
    }

    let mut cleaned_args = Vec::new();
    for i in 0..args.len() {
        if args[i] != "I" && args[i] != "am" && args[i] != "AM"{
            cleaned_args.push(args[i]);
        }
    }

    if cleaned_args.len() == 0 {
        return format!("WHAT IS YOU SAY MOTHER FUCKER.")
    }

    let mut desc = String::new();
    for i in 0..cleaned_args.len() {
        desc.push_str(&format!("{}{}", cleaned_args[i], if i == cleaned_args.len()-1 {""} else {" "}));
    }
    characters.get_mut(source_nickname).unwrap().description = desc;

    format!("{}, I agree completely.", source_nickname)
}
/// !a <item> is <desc>
fn a(source_nickname: &str, args: &Vec<&str>, items: &mut HashMap<String, Item>, characters: &HashMap<String, Character>) -> String {
    describe_item(source_nickname, args, items, characters)
}
/// !an <item> is <desc>
fn an(source_nickname: &str, args: &Vec<&str>, items: &mut HashMap<String, Item>, characters: &HashMap<String, Character>) -> String {
    describe_item(source_nickname, args, items, characters)
}

/// <item> is <desc>
fn describe_item(source_nickname: &str, args: &Vec<&str>, items: &mut HashMap<String, Item>, characters: &HashMap<String, Character>) -> String {
    if args.len() < 3 {
        return format!("is wat.")
    }

    let mut is_pos = 0;
    for i in 0..args.len() {
        if args[i] == "is" {
            is_pos = i;
            break;
        }
    }

    if is_pos == 0 {
        return format!("{}, stop making retarded item names.", source_nickname)
    }

    let item_name = get_item_name(&args[0..is_pos]);
    if !items.contains_key(&item_name) {
        return format!("that's bullshit.")
    }
    let mut desc = String::new();
    for i in (is_pos + 1)..args.len() {
        desc.push_str(&format!("{}{}", args[i], if i == args.len()-1 {""} else {" "}));
    }

    items.get_mut(&item_name).unwrap().description = desc;

    return format!("Interesting. {} have always fascinated me, {}.", &item_name, source_nickname);
}

/// For describing instances?
/*fn the(source_nickname: &str, args: &Vec<&str>, items: &HashMap<String, Item>, characters: &HashMap<String, Character>) -> String {
    format!("Unimplemented")
}*/

/// Equip the current item? Is this even something we want?
fn equip(source_nickname: &str, args: &Vec<&str>, items: &HashMap<String, Item>, characters: &HashMap<String, Character>) -> String {
    format!("Unimplemented")
}

/// Gives the item from you to the target character
// give <item> to <character>
// give <character> my <item>

//TODO: Fix this, it's shit.
fn give(source_nickname: &str, args: &Vec<&str>, items: &HashMap<String, Item>, characters: &mut HashMap<String, Character>) -> String {
    let mut cleaned_args = Vec::new();
    for i in 0..args.len() {
        if args[i] != "my" && args[i] != "MY" && args[i] != "a" && args[i] != "A" && args[i] != "an" && args[i] != "AN" {
            cleaned_args.push(args[i]);
        }
    }

    if cleaned_args.len() <= 1 {
        return format!("POR QUE ENGLAISE.")
    }

    let mut to_pos = 0;
    for i in 0..cleaned_args.len() {
        if cleaned_args[i] == "to" {
            to_pos = i;
            break;
        }
    }
    if to_pos == 0 {
        // using give 'character item' syntax
        let giver_name = source_nickname;
        let reciever_name = cleaned_args[0];
        let item_name = get_item_name(&cleaned_args[1..cleaned_args.len()]);

        {
            // Check that item exists
            if items.get(&item_name).is_none() {
                return format!("Don't start making shit up {}, {} don't exist.", giver_name, item_name)
            }

            // Check that the user isn't trying to be smarmy
            if reciever_name == "me" {
                return format!("Fuck you, get your own {}.", item_name)
            }

            // Check that giver exists
            let giver;
            if let Some(c) = characters.get(giver_name) {
                giver = c;
            } else {
                return format!("{}: who?", giver_name)
            }

            // Check that giver has the item
            if !giver.inventory.contains_key(&item_name) {
                return format!("pls, you don't even HAVE a {}!", item_name)
            }

            // Check that reciever exists
            let reciever;
            if let Some(c) = characters.get(reciever_name) {
                reciever = c;
            } else {
                return format!("{}: the fuck is a {}?", giver_name, item_name)
            }

            // Check that reciever's inventory slot isn't maxed out
            if let Some(count) = reciever.inventory.get(&item_name) {
                if *count == std::u64::MAX {
                    return format!("{} can't hold all of these {}!", reciever_name, item_name)
                }
            }
        }
        // By this point, we trust that all safety checks have been made, and begin mutating state.
        {
            // Get mutable reference to the giver
            let giver;
            if let Some(c) = characters.get_mut(giver_name) {
                giver = c;
            } else {
                return format!("{}: who?", giver_name)
            }

            // Remove the item from the giver
            let delete = match giver.inventory.get_mut(&item_name).unwrap() {
                &mut 1 => true,
                count => {
                    *count -= 1;
                    false
                }
            };
            if delete {
                giver.inventory.remove(&item_name);
            }
        }
        {
            // Get mutable reference to the reciever
            let reciever;
            if let Some(c) = characters.get_mut(reciever_name) {
                reciever = c;
            } else {
                return format!("{}: the fuck is a {}?", giver_name, item_name)
            }
            // Add the item to the reciever
            if reciever.inventory.contains_key(&item_name) {
                *reciever.inventory.get_mut(&item_name).unwrap() += 1;
            } else {
                reciever.inventory.insert(item_name.clone(), 1u64);
            }
        }
        return format!("Done!")

    } else {
        // using give 'item to character' syntax
    }
    return format!("W-what?")
}

fn bid(source_nickname: &str, args: &Vec<&str>, items: &HashMap<String, Item>, characters: &HashMap<String, Character>) -> String {
    format!("Unimplemented")

}

fn quit(source_nickname: &str, args: &Vec<&str>, items: &HashMap<String, Item>, characters: &HashMap<String, Character>) -> String {
    panic!("Quit.")
}

// string -> format string -> fn with the name of the string

// You can DESCRIBE yourself

// You can CREATE an item
// You can DESCRIBE an item IF you made it

// You can LOOK at an item
// You can LOOK at a user

// You can EQUIP an item
// You can GIVE an item to a user

// You can DROP an item on the floor
// You can GET an item on the floor

// You can BID on an item IF you didn't make it

/*
buy x
create x
describe x
*/

// I want a list of structs
// I don't know how many structs I want
// I don't know the type of structs
// I want them to be kept in sync with add and remove methods
// I want individual access to each field
/*
macro_rules! list {
    ( $struct_name: ident, $($field_name: ident: $field_type: ty,),+ ) => {
        struct $struct_name {
            $(
            field_name: field_type,
            )+
        }

        impl $struct_name {
            pub fn add($())
        }
    };
}*/