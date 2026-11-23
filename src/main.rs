use clearscreen::clear;
use colored::Colorize;
use rand::{random_bool, random_range};
use std::{env, error::Error, fmt, io, io::Write};
use strip_indent::strip_indent;

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
    Double,
    Split,
}

#[derive(Debug, PartialEq)]
enum RoundState {
    // The round is still going - the player is still hitting
    Ongoing,

    // Player stood - no other players to wait for, so we proceed to the showdown
    PlayerStand,

    // Player hit and went above 21
    PlayerBust,

    // Player has a blackjack, instant win
    PlayerBlackjack,

    // House has a blackjack, instant loss
    HouseBlackjack,

    // Both the player and the house have a blackjack, so the player gets the wager back
    BothBlackjack,

    // The player's sum was higher than the house's
    PlayerWin,

    // The house's sum was higher than the player's
    HouseWin,

    // While dealing cards up until 17, the house went abouve 21
    HouseBust,

    // Both the player and the house have the same hand sum
    Push,
}

#[derive(Debug, PartialEq)]
enum GameEndType {
    PlayerBankrupt,
    PlayerQuit,
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

/// Deals two cards into the given hand after emptying it.
fn deal_new_hand(hand: &mut Vec<Card>, used_cards: &mut Vec<Card>) {
    hand.clear();
    used_cards.clear();

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

/// Takes action input from the player for the current round, and returns the corresponding PlayerAction if it was valid and None otherwise.
fn take_action_input() -> Option<PlayerAction> {
    let mut input = String::new();

    while input == "" || !("hs".contains(input.as_str())) {
        print!("Enter action [h/s]: ");
        io::stdout().flush().unwrap();

        input.clear();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input!");

        input = String::from(input.trim().to_ascii_lowercase());
    }

    match input.as_str() {
        "h" => Some(PlayerAction::Hit),
        "s" => Some(PlayerAction::Stand),
        _ => None,
    }
}

/// Takes wager input from the player for the current round, and returns the wager. It harasses the player until they enter a valid wager.
fn take_wager_input(balance: i32) -> i32 {
    let mut wager: i32 = 0;
    let mut input: String = String::new();

    while wager > balance || wager < 1 {
        print!("Wager: ");
        io::stdout().flush().unwrap();

        input.clear();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input!");

        wager = match input.trim().parse() {
            Ok(w) => w,
            Err(_) => 0,
        };
    }

    wager
}

/// Waits for the player to press enter to continue.
fn await_enter() {
    print!("{}", "Press [ENTER] to continue...".white().dimmed());
    io::stdout().flush().unwrap();
    let mut press_enter_input = String::new();
    io::stdin().read_line(&mut press_enter_input).unwrap();
}

/// Shorthand for clearing the screen.
fn clear_screen() {
    clearscreen::clear().unwrap();
}

/// Shorthand for rendering and printing both the player and house hands.
fn print_both_hands(player_hand: &Vec<Card>, house_hand: &Vec<Card>, house_hole_card_hidden: bool) {
    // Tomfoolery
    let house_name = if random_bool(0.05) {
        "House M.D."
    } else {
        "House"
    };

    // Display hands
    println!("Your hand: {}", render_hand(&player_hand, None));
    println!(
        "{}'s hand: {}\n",
        house_name,
        render_hand(
            &house_hand,
            if house_hole_card_hidden {
                Some(vec![1])
            } else {
                None
            }
        )
    );
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
    let mut banner: String = String::from(
        "
   .------..------..------..------..------.     .------..------..------..------..------.
   |W.--. ||H.--. ||I.--. ||T.--. ||E.--. |.-.  |Q.--. ||U.--. ||E.--. ||E.--. ||N.--. |
   | :/\\: || :/\\: || (\\/) || :/\\: || (\\/) ((5)) | (\\/) || (\\/) || (\\/) || (\\/) || :(): |
   | :\\/: || (__) || :\\/: || (__) || :\\/: |'-.-.| :\\/: || :\\/: || :\\/: || :\\/: || ()() |
   | '--'W|| '--'H|| '--'I|| '--'T|| '--'E| ((1)) '--'Q|| '--'U|| '--'E|| '--'E|| '--'N|
   `------'`------'`------'`------'`------'  '-'`------'`------'`------'`------'`------'
    ",
    );

    for letter in "WHITEQUN".chars() {
        banner = banner.replace(
            letter,
            format!("{}", letter.to_string().magenta().bold()).as_str(),
        );
    }

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
    let mut current_wager: i32;

    let mut round_state: RoundState = RoundState::Ongoing;
    let mut is_game_ongoing: bool = true;

    // Welcome
    clear_screen();
    println!("{}", banner);

    println!("{}{}{}", "Welcome to ", "WHITE QUEEN".magenta().bold(), "!");
    println!(
        "{}{}{}",
        "Your bankroll is ",
        format!("{} Doubloons", bankroll).cyan().bold(),
        ".\n"
    );

    await_enter();

    // Game loop (until exit)
    'game_ongoing: while is_game_ongoing {
        // Reset round state
        round_state = RoundState::Ongoing;

        // Clear & title
        clear_screen();
        println!("{}", banner);

        // Print balance
        println!(
            "Balance: {}",
            format!("{} Doubloons", balance).cyan().bold() // TODO: Display most recent win/loss (e.g. +500 in green or -300 in red)
        );

        // Take wager input
        current_wager = take_wager_input(balance);
        balance -= current_wager;

        // Deal hands
        deal_new_hand(&mut player_hand, &mut used_cards);
        deal_new_hand(&mut house_hand, &mut used_cards);

        // * TEMP: Rig hands to test blackjack matching
        //         player_hand[0] = Card {
        //             rank: 10,
        //             suit: Suit::Hearts,
        //         };
        //
        //         player_hand[1] = Card {
        //             rank: 1,
        //             suit: Suit::Clubs,
        //         };
        //
        //         house_hand[0] = Card {
        //             rank: 10,
        //             suit: Suit::Diamonds,
        //         };
        //
        //         house_hand[1] = Card {
        //             rank: 1,
        //             suit: Suit::Spades,
        //         };

        // TODO: Add double (introduce new variable `has_hit` and check if it's false) (not `hit_count: i8` because a bool is more memory-efficient)

        // Update round state for player/house/both blackjack
        match (get_hand_sum(&player_hand), get_hand_sum(&house_hand)) {
            (21, 21) => round_state = RoundState::BothBlackjack,
            (21, _) => round_state = RoundState::PlayerBlackjack,
            (_, 21) => round_state = RoundState::HouseBlackjack,
            _ => (),
        }

        // Handle player blackjack, house blackjack, and push
        if vec![
            RoundState::BothBlackjack,
            RoundState::PlayerBlackjack,
            RoundState::HouseBlackjack,
        ]
        .contains(&round_state)
        {
            // Clear & title
            clear_screen();
            println!("{}", banner);

            match round_state {
                RoundState::PlayerBlackjack => {
                    balance += current_wager * 2;

                    println!("{}", "You have a blackjack!".green().bold());
                    println!(
                        "You won {}, instantly getting back double your wager!",
                        format!("{} Doubloons", current_wager).cyan().bold()
                    );
                }
                RoundState::HouseBlackjack => {
                    println!("{}", "The house has a blackjack!".red().bold());
                    println!(
                        "You lost your wager of {} instantly.",
                        format!("{} Doubloons", current_wager).cyan().bold()
                    );
                }
                RoundState::BothBlackjack => {
                    balance += current_wager;

                    println!(
                        "{}",
                        "It's a push! Both you and the house have blackjacks."
                            .yellow()
                            .bold()
                    );
                    println!(
                        "You instantly got back your wager of {}.",
                        format!("{} Doubloons", current_wager).cyan().bold()
                    );
                }
                _ => (),
            }

            println!();
            print_both_hands(&player_hand, &house_hand, false);

            await_enter();
        }

        // Hitting loop (round loop)
        'round_ongoing: while round_state == RoundState::Ongoing {
            // Clear & title
            clear_screen();
            println!("{}", banner);

            // Display balance & wager
            println!(
                "Balance: {}",
                format!("{} Doubloons", balance).cyan().bold(),
            );

            println!(
                "Wager: {}\n",
                format!("{} Doubloons", current_wager).yellow().bold(),
            );

            print_both_hands(&player_hand, &house_hand, true);

            // Take input
            match take_action_input() {
                Some(PlayerAction::Hit) => {
                    player_hand.push(generate_card(&mut used_cards));
                    println!("{}", render_hand(&player_hand, None));

                    // Bust
                    if get_hand_sum(&player_hand) > 21 {
                        round_state = RoundState::PlayerBust;
                    } else {
                        continue 'round_ongoing;
                    }
                }
                Some(PlayerAction::Stand) => {
                    round_state = RoundState::PlayerStand;
                }
                _ => (),
            }

            // Showdown
            clear_screen();
            println!("{}", banner);
            println!("{}", "ROUND OVER!".magenta().bold());

            // Start dealing cards for house
            // TODO: If house busts, player gets double the wager back

            // TODO: If equal, player gets the wager back
            // TODO: If house's hand is higher, do nothing (player loses wager)

            dbg!(&round_state);

            println!(
                "@@@ Your sum: {}, house sum: {}",
                get_hand_sum(&player_hand),
                get_hand_sum(&house_hand)
            );

            // Prompt for new round
            println!();

            let mut another_round_input: String = String::new();
            while another_round_input.to_lowercase() != "y"
                || another_round_input.to_lowercase() != "n"
            {
                print!("Another round? [Y/n]: ");
                io::stdout().flush().unwrap();

                io::stdin()
                    .read_line(&mut another_round_input)
                    .expect("Failed to read input!");

                another_round_input = String::from(another_round_input.trim());
                match another_round_input.to_lowercase().as_str() {
                    "y" | "" => {
                        round_state = RoundState::Ongoing;
                        break 'round_ongoing;
                    }
                    "n" => {
                        // TODO: Update game exit type, handle exit at the bottom
                        println!("@@@ user exited");
                        is_game_ongoing = false;

                        break 'round_ongoing;
                    }
                    _ => (),
                };
            }
        }
    }
}
