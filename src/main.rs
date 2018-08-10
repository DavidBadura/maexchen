extern crate rand;
use rand::Rng;

use std::cmp::Ordering;
use std::net::UdpSocket;
use std::str;

fn main() {
    let server = Server {
        socket: UdpSocket::bind("0.0.0.0:34000").expect("couldn't bind to address"),
        address: String::from("192.168.179.27:9000"),
    };

    server.send(format!("REGISTER;Amazing"));

    let mut buf = [0; 2048];
    let mut game: Game = Game { round_number: 0, dices: vec![] };
    loop {
        match server.socket.recv_from(&mut buf) {
            Ok((number_of_bytes, _src)) => {
                let filled_buf = &mut buf[..number_of_bytes];

                let message = str::from_utf8(&filled_buf).unwrap_or("");

                let arguments: Vec<&str> = message.split(";").collect();
                let command = arguments[0];

                println!("< {}", message);


                match command {
                    "ROUND STARTING" => {
                        join(&server, String::from(arguments[1]));
                    }
                    "ROUND STARTED" => {
                        game = Game { round_number: 1, dices: vec![] };
                    }
                    "YOUR TURN" => {
                        turn(&server, String::from(arguments[1]), game.clone());
                    }
                    "ROLLED" => {
                        let dice: Dice = dice_from_string(String::from(arguments[1]));

                        rolled(&server, String::from(arguments[2]), game.clone(), dice);
                    }
                    "ANNOUNCED" => {
                        game.dices.push(dice_from_string(String::from(arguments[2])));
                    }
                    _ => {}
                }
            }
            Err(e) => {
                println!("couldn't recieve a datagram: {}", e);
            }
        }
    }
}


fn join(server: &Server, token: String) {
    server.send(format!("JOIN;{}", token));
    println!("> Joined!");
}

fn turn(server: &Server, token: String, game: Game) {
    if game.dices.len() == 0 {
        server.send(format!("ROLL;{}", token));

        return;
    }

    let dice = game.dices.last().unwrap().clone();

    /*
    let number = dice.to_int();

    if number == 66 || number == 55 || number == 44 {
        server.send(format!("SEE;{}", token));

        return;
    }

    if game.dices.len() > 1 {
        let before = game.dices[game.dices.len() - 2].clone();

        if before.next() == dice {
            server.send(format!("SEE;{}", token));

            return;
        }
    }
    */

    let num = rand::thread_rng().gen_range(0, 100);

    if dice.probability() + 25 < num {
        server.send(format!("SEE;{}", token));

        return;
    }

    server.send(format!("ROLL;{}", token));
    println!("> Turned!");
}

fn rolled(server: &Server, token: String, game: Game, dice: Dice) {
    if game.dices.len() == 0 {
        for _x in 0..10 {
            let new_dice = random_dice();

            if new_dice > dice && !new_dice.is_maexchen() && !new_dice.is_doubles() {
                server.send(format!("ANNOUNCE;{};{}", new_dice.to_string(), token));

                return;
            }
        }

        server.send(format!("ANNOUNCE;{};{}", dice.to_string(), token));

        return;
    }

    let last_dice = game.dices.last().unwrap().clone();

    println!("{:?}", last_dice);

    if dice > last_dice {
        server.send(format!("ANNOUNCE;{};{}", dice.to_string(), token));
    } else {
        for _x in 0..10 {
            let new_dice = random_dice();

            if new_dice > last_dice && !new_dice.is_maexchen() && !new_dice.is_doubles() {
                server.send(format!("ANNOUNCE;{};{}", new_dice.to_string(), token));

                return;
            }
        }

        server.send(format!("ANNOUNCE;{};{}", last_dice.next().to_string(), token));
    }

    println!("> Announced!");
}


/* Server */

#[derive(Debug)]
struct Server {
    socket: UdpSocket,
    address: String,
}


impl Server {
    fn send(&self, message: String) {
        let a = self.address.clone();
        self.socket.send_to(message.as_bytes(), a).expect("could't send");
    }
}

/* GAME */

#[derive(Debug, Clone)]
struct Game {
    round_number: u32,
    dices: Vec<Dice>,
}


/* DICE */

fn dice_from_string(string: String) -> Dice {
    let pair: Vec<&str> = string.split(",").collect();

    return Dice { d1: pair[0].parse::<u32>().unwrap(), d2: pair[1].parse::<u32>().unwrap() };
}


fn random_dice() -> Dice {
    return Dice {
      d1: rand::thread_rng().gen_range(1, 6),
      d2: rand::thread_rng().gen_range(1, 6),
    };
}


fn bigger_random_dice(before: Dice) -> Dice {
    loop {
        let new = random_dice();

        if new > before {
            return new;
        }
    }
}

#[derive(Debug, Eq, Clone)]
struct Dice {
    d1: u32,
    d2: u32,
}


impl Dice {
    fn to_string(&self) -> String {
        return format!("{},{}", self.d1, self.d2);
    }

    fn to_int(&self) -> u32 {
        return self.d1 * 10 + self.d2;
    }

    fn is_doubles(&self) -> bool {
        return self.d1 == self.d2;
    }

    fn is_maexchen(&self) -> bool {
        return self.d1 == 2 && self.d2 == 1;
    }

    fn next(&self) -> Dice {
        if self.is_maexchen() {
            return Dice { d1: self.d1, d2: self.d2 };
        }

        if self.d1 == 6 && self.d2 == 6 {
            return Dice { d1: 2, d2: 1 };
        }

        if self.is_doubles() {
            return Dice {
                d1: self.d1 + 1,
                d2: self.d2 + 1,
            };
        }

        if self.d1 == 6 && self.d2 == 5 {
            return Dice { d1: 1, d2: 1 };
        }

        if self.d2 == 6 {
            return Dice {
                d1: self.d1 + 1,
                d2: 1,
            };
        }

        let new = Dice {
            d1: self.d1,
            d2: self.d2 + 1,
        };

        if !new.is_doubles() {
            return new;
        }

        return Dice {
            d1: new.d1,
            d2: new.d2 + 1,
        };
    }

    fn probability(&self) -> u32 {
        match self.clone() {
            Dice { d1: 3, d2: 1 } => { return 100; }
            Dice { d1: 3, d2: 2 } => { return 94; }
            Dice { d1: 4, d2: 1 } => { return 88; }
            Dice { d1: 4, d2: 2 } => { return 83; }
            Dice { d1: 4, d2: 3 } => { return 77; }
            Dice { d1: 5, d2: 1 } => { return 72; }
            Dice { d1: 5, d2: 2 } => { return 66; }
            Dice { d1: 5, d2: 3 } => { return 61; }
            Dice { d1: 5, d2: 4 } => { return 55; }
            Dice { d1: 6, d2: 1 } => { return 50; }
            Dice { d1: 6, d2: 2 } => { return 44; }
            Dice { d1: 6, d2: 3 } => { return 39; }
            Dice { d1: 6, d2: 4 } => { return 33; }
            Dice { d1: 6, d2: 5 } => { return 28; }
            Dice { d1: 1, d2: 1 } => { return 22; }
            Dice { d1: 2, d2: 2 } => { return 19; }
            Dice { d1: 3, d2: 3 } => { return 17; }
            Dice { d1: 4, d2: 4 } => { return 14; }
            Dice { d1: 5, d2: 5 } => { return 11; }
            Dice { d1: 6, d2: 6 } => { return 8; }
            Dice { d1: 2, d2: 1 } => { return 6; }
            _ => { return 0; }
        }
    }
}

impl Ord for Dice {
    fn cmp(&self, other: &Dice) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }

        if self.is_maexchen() {
            return Ordering::Greater;
        }

        if other.is_maexchen() {
            return Ordering::Less;
        }

        if self.is_doubles() {
            if other.is_doubles() {
                if self.to_int() > other.to_int() {
                    return Ordering::Greater;
                }

                return Ordering::Less;
            }

            return Ordering::Greater;
        }

        if other.is_doubles() {
            return Ordering::Less;
        }

        if self.to_int() > other.to_int() {
            return Ordering::Greater;
        }

        return Ordering::Less;
    }
}


impl PartialOrd for Dice {
    fn partial_cmp(&self, other: &Dice) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Dice {
    fn eq(&self, other: &Dice) -> bool {
        return self.d1 == other.d1 && self.d2 == other.d2;
    }
}
