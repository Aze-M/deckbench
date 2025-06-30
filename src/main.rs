use hex;
use rand;
use std::thread::{self, JoinHandle};
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default)]
pub struct Card {
    pub suit: u8,
    pub rank: u8,
}

#[allow(unused_parens)]
fn main() {
    //threading
    let total_cores: usize = thread::available_parallelism().unwrap().into();
    let total_cores = total_cores as u128;

    loop {
        //init
        let mut input = String::new();
        let mut valid = false;
        let mut active = true;

        while (!valid) {
            println!("How many decks to make? (empty for exit)");
            std::io::stdin().read_line(&mut input).unwrap();
            input = input.chars().filter(|x| x.is_numeric()).collect();

            valid = input.parse::<u32>().is_ok();

            if (input == "") {
                active = false;
                valid = true;
            }
        }

        if !active {
            break;
        };

        let amt = input.parse::<u128>().unwrap();

        if (amt % total_cores as u128 > 0) {
            println!(
                "Rounding to {:?} for clean work division between cores.",
                (amt / total_cores) * total_cores
            )
        }

        println!(
            "Creating and Hashing {:?} Decks with {:?} threads.",
            amt, total_cores
        );

        let amt_per_thread = amt / total_cores;

        let starttime = Instant::now();
        let mut threads: Vec<JoinHandle<()>> = vec![];

        for _ in 0..total_cores {
            threads.push(thread::spawn(move || {
                let mut deck = make_deck();

                let mut bytebuffer: [u8; 39] = [0; 39];
                for _ in 0..amt_per_thread {
                    shuffle_deck(&mut deck);
                    bytebuffer.fill(0);
                    make_deck_hash(deck, &mut bytebuffer);
                    //println!("{:?}", make_deck_hash(deck, &mut bytebuffer));
                }
                return;
            }));
        }

        for handle in threads {
            handle.join().unwrap();
        }

        let endtime = Instant::now();
        println!(
            "Created an Hashed {:?} Decks in {:.2} seconds.",
            amt_per_thread * total_cores,
            endtime.duration_since(starttime).as_secs_f32()
        );
    }
}

fn shuffle_deck(deck: &mut [Card; 52]) {
    let mut buffer: Card;

    for x in 0..51 {
        let idx = x as usize;
        let id2: usize = rand::random_range(idx..52);

        buffer = deck[id2];
        deck[id2] = deck[idx];
        deck[idx] = buffer;
    }
}

#[allow(unused_parens)]
fn make_deck_hash(deck: [Card; 52], bytebuffer: &mut [u8; 39]) -> String {
    let mut bitshifter: u8 = 0;
    let mut byteshifter: usize = 0;

    for x in 0..52 {
        let suit = deck[x].suit;
        let rank = deck[x].rank;

        bytebuffer[byteshifter] |= suit << bitshifter;
        bitshifter += 2;

        if (bitshifter <= 4) {
            bytebuffer[byteshifter] |= rank << bitshifter;
            bitshifter += 4;
        } else {
            // shifter is 6 or higher
            bytebuffer[byteshifter] |= (rank.checked_shl(bitshifter as u32).unwrap_or(0)) & 0xFF;
            bytebuffer[byteshifter + 1] |= rank >> (8 - bitshifter);
            bitshifter += 4;
        }

        if (bitshifter >= 8) {
            bitshifter -= 8;
            byteshifter += 1;
        }
    }

    let hash = hex::encode(bytebuffer);
    return hash;
}

#[allow(unused_parens)]
fn make_deck() -> [Card; 52] {
    let mut deck: [Card; 52] = [Card::default(); 52];
    let mut suitinc: u8 = 0;
    let mut rankinc: u8 = 0;

    for x in 0..52 {
        deck[x] = Card {
            suit: suitinc,
            rank: rankinc,
        };
        rankinc += 1;
        if (rankinc == 13) {
            rankinc = 0;
            suitinc += 1;
        }
    }

    return deck;
}
