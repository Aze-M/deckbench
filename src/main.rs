use hex;
use rand;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;
use std::thread::{self, JoinHandle};
use std::time::Instant;
use std::{env, fs::File, path::Path, u128, vec};

#[derive(Debug, Clone, Copy, Default)]
pub struct Card {
    pub suit: u8,
    pub rank: u8,
}

#[allow(unused_parens)]
fn main() {
    //threading
    let mut total_cores = thread::available_parallelism().unwrap().get() as u128;
    let mut amtarg: Vec<&String> = vec![];

    //init
    let mut active = true;
    let mut res_only = false;
    let mut log = false;

    //arg wrapping
    let args: Vec<String> = env::args().collect();

    if (args.contains(&"res-only".to_string())) {
        res_only = true;
    }

    if (args.contains(&"no-mt".to_string())) {
        if (!res_only) {
            println!("Disabling Multi-threading.");
        }
        total_cores = 1;
    }

    if (args.contains(&"log".to_string())) {
        log = true;
    }

    if (args.len() > 1) {
        amtarg = args.iter().filter(|x| x.parse::<u128>().is_ok()).collect();
    }

    loop {
        if !active {
            break;
        };
        //init loop resets
        let mut input = String::new();
        let mut valid = false;

        while (!valid && amtarg.len() < 1) {
            println!("How many decks to make? (empty for exit)");
            std::io::stdin().read_line(&mut input).unwrap();
            input = input.chars().filter(|x| x.is_numeric()).collect();

            valid = input.parse::<u32>().is_ok();

            if (input.is_empty()) {
                active = false;
                valid = true;
            }
        }

        if (amtarg.len() >= 1) {
            input = amtarg[0].to_owned();
            active = false;
        }

        let amt: u128 = match input.parse::<u128>() {
            Err(reason) => {
                println!("Could not unwrap input amount: {:?}", reason);
                exit(0);
            }
            Ok(result) => result,
        };

        if (amt % total_cores as u128 > 0) {
            if (!res_only) {
                println!(
                    "Rounding to {:?} for clean work division between cores.",
                    (amt / total_cores) * total_cores
                )
            }
        }

        if (!res_only) {
            println!(
                "Creating deck per thread, Shuffling and Hashing {:?} total times with {:?} threads.",
                amt, total_cores
            );
        }

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
                    let _ = make_deck_hash(deck, &mut bytebuffer);
                    //println!("{:?}", make_deck_hash(deck, &mut bytebuffer));
                }
                return;
            }));
        }

        for handle in threads {
            handle.join().unwrap();
        }

        let endtime = Instant::now();
        if (!res_only) {
            println!(
                "Shuffled an Hashed {:?} times in {:.2} seconds.",
                amt_per_thread * total_cores,
                endtime.duration_since(starttime).as_secs_f32()
            );
        } else {
            println!(
                "{:?};{:.2}s",
                amt_per_thread * total_cores,
                endtime.duration_since(starttime).as_secs_f32()
            );
        }

        if (log) {
            let path = Path::new("output.log");
            let mut file = match OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path)
            {
                Err(_) => File::create(path).unwrap(),
                Ok(file) => file,
            };

            let entry = format!(
                "{:?};{:.2}s\n",
                amt_per_thread * total_cores,
                endtime.duration_since(starttime).as_secs_f32()
            );
            file.write_all(entry.as_bytes()).unwrap_or_default();
        }
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
