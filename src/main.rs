use colored::Colorize;
use rand::random_range;
use std::env;

#[derive(Debug, PartialEq, Clone)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, PartialEq, Clone)]
struct Card {
    suit: Suit,
    rank: u8,
}

const STARTING_BET_HINT: &str = "Your starting bet must be an integer above 0! Pass it as an argument to the white-queen executable.";

fn main() {
    let args: Vec<String> = env::args().collect();
    let starting_bet: i32 = validate_starting_bet_argument(args);
    if starting_bet == -1 {
        // Validation failed
        return;
    };

    // Initial variables
    let mut player_hand: Vec<Card> = Vec::new();
    let mut house_hand: Vec<Card> = Vec::new();
    let mut used_cards: Vec<Card> = Vec::new();

    // Welcome
    clearscreen::clear().unwrap();

    println!("{}{}{}", "Welcome to ", "WHITE QUEEN".magenta().bold(), "!");
    println!(
        "{}{}{}",
        "Your starting bet is ",
        format!("{} Doubloon(s)", starting_bet).cyan().bold(),
        "."
    );

    // Deal hands
    deal_hand(&mut player_hand, &mut used_cards);
    deal_hand(&mut house_hand, &mut used_cards);

    // TEMP
    println!("Player hand sum: {}", get_hand_sum(&player_hand));
    println!("House hand sum: {}", get_hand_sum(&house_hand));

    println!("{}", player_hand[0].render());
}

/// Returns -1 if the validation failed, and the starting bet otherwise.
///
/// The validation can fail for one of three reasons:
/// 1. The player entered no command line argument.
/// 2. The player entered a negative starting bet.
/// 3. The player entered 0 as the starting bet.
fn validate_starting_bet_argument(args: Vec<String>) -> i32 {
    if args.len() < 2 {
        println!("{}", STARTING_BET_HINT.red());
        return -1;
    }

    let starting_bet: i32;
    match args[1].trim().parse::<i32>() {
        Err(_) => {
            println!("{}", STARTING_BET_HINT.red());
            return -1;
        }
        Ok(num) => {
            if num <= 0 {
                println!("{}", STARTING_BET_HINT.red());
                return -1;
            } else {
                starting_bet = num;
                return starting_bet;
            }
        }
    }
}

/// Can be used to deal cards for both the player and the house.
fn deal_hand(hand: &mut Vec<Card>, used_cards: &mut Vec<Card>) {
    let new_card_1 = generate_card(used_cards);
    let new_card_2 = generate_card(used_cards);

    hand.push(new_card_1);
    hand.push(new_card_2);
}

/// Returns a card with a random suit and rank.
fn generate_card(used_cards: &mut Vec<Card>) -> Card {
    let mut new_card: Card;

    loop {
        new_card = Card {
            rank: random_range(1..=13),
            suit: get_random_suit(),
        };
        if !used_cards.contains(&new_card) {
            used_cards.push(new_card.clone());
            break;
        }
    }

    new_card
}

/// Returns the sum of the cards in the hand.
fn get_hand_sum(hand: &Vec<Card>) -> u8 {
    let mut sum: u8 = 0;
    let mut has_ace: bool = false;

    for card in hand.iter() {
        sum += evaluate_card(card);
        if card.rank == 1 {
            has_ace = true;
        }
    }

    // If there's an ace in the hand, and it would bust, then flip that ace
    // to only be worth 1. This will only work on a single ace - if it still
    // busts, that should be checked and handled wherever this function is used.
    if sum > 21 && has_ace {
        sum -= 10;
    }

    sum
}

/// Returns the true value of a given card, to be used in the sum calculation.
///
/// All face cards evaluate to 10, and aces evaluate to 11 by default.
fn evaluate_card(card: &Card) -> u8 {
    match card.rank {
        1 => 11,
        11 | 12 | 13 => 10,
        _ => card.rank,
    }
}

fn get_random_suit() -> Suit {
    match random_range(0..=3) {
        0 => Suit::Clubs,
        1 => Suit::Diamonds,
        2 => Suit::Hearts,
        _ => Suit::Spades,
    }
}

fn render_rank(rank: u8) -> String {
    return match rank {
        1 => String::from("A"),
        11 => String::from("J"),
        12 => String::from("Q"),
        13 => String::from("K"),
        _ => format!("{rank}"),
    };
}

impl Card {
    fn render(&self) -> String {
        let symbol = match &self.suit {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        };

        let uncolored: String = format!("{} {}", symbol, render_rank(self.rank));

        match &self.suit {
            Suit::Diamonds | Suit::Hearts => return format!("{}", uncolored.red()),
            Suit::Clubs | Suit::Spades => return format!("{}", uncolored.blue()),
        }
    }
}
