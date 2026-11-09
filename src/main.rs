use colored::Colorize;
use rand::{random_bool, random_range};
use std::{env, error::Error, fmt, io, io::Write};

#[derive(Debug)]
enum InvalidArgError {
    NoArgument,
    NonNumberArgument,
    NonPositiveIntArgument,
}

#[derive(Debug, PartialEq)]
enum PlayerAction {
    Hit,
    Stand,
    // TODO: double, split
}

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

/// Returns whether the command-line argument provided by the player as the bankroll was valid.
fn validate_bankroll_argument(args: Vec<String>) -> Result<i32, InvalidArgError> {
    if args.len() < 2 {
        return Err(InvalidArgError::NoArgument);
    }

    let bankroll: i32;
    match args[1].trim().parse::<i32>() {
        Err(_) => {
            return Err(InvalidArgError::NonNumberArgument);
        }
        Ok(num) => {
            if num <= 0 {
                return Err(InvalidArgError::NonPositiveIntArgument);
            } else {
                bankroll = num;
                return Ok(bankroll);
            }
        }
    }
}

/// Deals two cards into the given hand.
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

/// Returns the true value of a given card. All face cards evaluate to 10, and aces evaluate to 11 by default.
fn evaluate_card(card: &Card) -> u8 {
    match card.rank {
        1 => 11,
        11 | 12 | 13 => 10,
        _ => card.rank,
    }
}

/// Returns a random suit.
fn get_random_suit() -> Suit {
    match random_range(0..=3) {
        0 => Suit::Clubs,
        1 => Suit::Diamonds,
        2 => Suit::Hearts,
        _ => Suit::Spades,
    }
}

/// Returns the number or letter for the given rank (A for aces, J, Q and K for face cards).
fn render_rank(rank: u8) -> String {
    return match rank {
        1 => String::from("A"),
        11 => String::from("J"),
        12 => String::from("Q"),
        13 => String::from("K"),
        _ => format!("{rank}"),
    };
}

/// Returns a formatted sequence of the provided hand, optionally with the cards at the provided indices being hidden.
fn render_hand(hand: &Vec<Card>, hidden_indices: Option<Vec<usize>>) -> String {
    let mut sum: u8 = get_hand_sum(hand);

    let hidden_indices: Vec<usize> = match hidden_indices {
        Some(v) => v,
        None => Vec::new(),
    };

    let mut rendered_cards: Vec<String> = Vec::new();
    for (i, card) in hand.iter().enumerate() {
        if hidden_indices.contains(&i) {
            rendered_cards.push(format!("{}", "??".magenta()));
            sum -= evaluate_card(&hand[i]);
        } else {
            rendered_cards.push(card.render(false));
        }
    }

    format!(
        "{} {}",
        rendered_cards.join(", "),
        format!(
            "({sum}{})",
            if !hidden_indices.is_empty() { "?" } else { "" }
        )
        .dimmed()
    )
}

/// Takes input from the player for the current round, and returns the corresponding PlayerAction if it was valid and None otherwise.
fn take_input() -> Option<PlayerAction> {
    let mut player_input = String::new();

    print!("Enter action (h/s): ");
    let _ = io::stdout().flush();

    io::stdin()
        .read_line(&mut player_input)
        .expect("Failed to read input!");

    player_input = String::from(player_input.trim().to_ascii_lowercase());

    match player_input.as_str() {
        "h" => Some(PlayerAction::Hit),
        "s" => Some(PlayerAction::Stand),
        _ => None,
    }
}

impl Card {
    fn render(&self, hidden: bool) -> String {
        if hidden {
            return format!("{}", "??".magenta());
        }

        let uncolored: String = format!("{} {}", self.suit.symbol(), render_rank(self.rank));

        match &self.suit {
            Suit::Diamonds | Suit::Hearts => return format!("{}", uncolored.red()),
            Suit::Clubs | Suit::Spades => return format!("{}", uncolored.blue()),
        }
    }
}

impl Suit {
    fn symbol(&self) -> &str {
        match &self {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        }
    }
}

impl Error for InvalidArgError {}
impl fmt::Display for InvalidArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidArgError::NoArgument => {
                write!(f, "No argument was provided as the bankroll!")
            }
            InvalidArgError::NonNumberArgument => {
                write!(f, "The argument provided as the bankroll was not a number!")
            }
            InvalidArgError::NonPositiveIntArgument => {
                write!(
                    f,
                    "The argument provided as the bankroll was not a positive integer!"
                )
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Validate argument input
    let bankroll: i32;
    match validate_bankroll_argument(args) {
        Ok(bet) => bankroll = bet,
        Err(e) => {
            println!("{}", e.to_string().red());
            return;
        }
    }

    // Initial variables
    let mut player_hand: Vec<Card> = Vec::new();
    let mut house_hand: Vec<Card> = Vec::new();
    let mut used_cards: Vec<Card> = Vec::new();

    let mut balance: i32 = bankroll;
    let mut current_wager: i32 = 0;

    let mut is_round_ongoing: bool = true;
    let mut is_game_ongoing: bool = true;

    // Welcome
    clearscreen::clear().unwrap();

    println!("{}{}{}", "Welcome to ", "WHITE QUEEN".magenta().bold(), "!");
    println!(
        "{}{}{}",
        "Your bankroll is ",
        format!("{} Doubloon(s)", bankroll).cyan().bold(),
        "."
    );

    // Deal hands
    deal_hand(&mut player_hand, &mut used_cards);
    deal_hand(&mut house_hand, &mut used_cards);

    // TODO: Main game loop
    // TODO: Check house blackjack right after deal

    // Game loop (until exit)
    while is_game_ongoing {
        // TODO: Ask for round wager

        // Hitting loop (round loop)
        while is_round_ongoing {
            clearscreen::clear().unwrap();
            println!(
                "Balance: {}\n", // TODO: Also display current wager
                format!("{} Doubloon(s)", balance).cyan().bold()
            );

            let house_name = if random_bool(0.05) {
                "House M.D."
            } else {
                "House"
            };

            println!("Your hand: {}", render_hand(&player_hand, None));
            println!(
                "{}'s hand: {}\n",
                house_name,
                render_hand(&house_hand, Some(vec![1]))
            );

            if get_hand_sum(&house_hand) == 21 {
                is_round_ongoing = false;
            }

            // Take input
            match take_input() {
                Some(PlayerAction::Hit) => {
                    player_hand.push(generate_card(&mut used_cards));
                }
                Some(PlayerAction::Stand) => is_round_ongoing = false,
                None => balance += 7000, // TODO: Change this beauty
            }
        }

        // Showdown
        println!("@@@ --- SHOWDOWN ---");

        // TODO: Start dealing cards for house
        // TODO: If house busts, player gets double the wager back
        // TODO: If equal, player gets the wager back
        // TODO: If house's hand is higher, do nothing (player loses wager)

        if get_hand_sum(&house_hand) == 21 {
            // TODO: Change this (message, + add wager loss logic)
            // TODO: Player keeps wager if they have blackjack as well
            println!("House has blackjack! Fuck you");
        }

        println!(
            "Your sum: {}, house sum: {}",
            get_hand_sum(&player_hand),
            get_hand_sum(&house_hand)
        );
    }
}
