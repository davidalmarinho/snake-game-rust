extern crate termios;

use std::{thread, time};
use std::io::Write;
use std::io::{self, Read};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use rand::Rng;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};

const WIDTH: i16 = 50;
const HEIGHT: i16 = 20;

fn print_board(p_pos: &mut Vec<i16>, rocks_pos: &mut Vec<i16>) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut print_p: bool = false;
            let mut print_rock : bool = false;
            for index in 0..p_pos.len() {
                if x + y * WIDTH == p_pos[index] {
                    print_p = true;
                } else if x + y * WIDTH == rocks_pos[index] {
			print_rock = true;
	        }
            }
            if print_p {
                print!("🐍");
            } else if print_rock {
		print!("🪨");
            } else {
                print!("🌱");
            }
        }
        println!("");
    }
}

fn refresh() {
    print!("{}c", 27 as char);
    std::io::stdout().flush().unwrap();
}

fn read_input() -> char {
    let stdin = 0; // couldn't get std::os::unix::io::FromRawFd to work 
                   // on /dev/stdin or /dev/tty
    let termios = Termios::from_fd(stdin).unwrap();
    let mut new_termios = termios.clone();  // make a mutable copy of termios 
                                            // that we will modify
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0;1];  // read exactly one byte
    // print!("Hit a key! ");
    stdout.lock().flush().unwrap();
    reader.read_exact(&mut buffer).unwrap();
    // println!("You have hit: {:?}", buffer[0]);
    tcsetattr(stdin, TCSANOW, & termios).unwrap();  // reset the stdin to 
                                                    // original termios data
    buffer[0] as char
}

fn sleep(time_milis: u64) {
    let duration = time::Duration::from_millis(time_milis);
    thread::sleep(duration);
}

fn spawn_stdin_channel() -> Receiver<char> {
    let (tx, rx) = mpsc::channel::<char>();
    thread::spawn(move || loop {
        let buffer = read_input();
        tx.send(buffer).unwrap();
    });
    rx
}

fn snake_move(cur_play: char, p_pos: &mut Vec<i16>) {
    // Move right
    if cur_play == 'd' {
        let mut next_pos = p_pos[p_pos.len() - 1] + 1;
        if next_pos % WIDTH == 0 {
            next_pos -= WIDTH;
        }
        p_pos.push(next_pos);
        p_pos.remove(0);
    // Move left
    } else if cur_play == 'a' {
        let mut next_pos = p_pos[p_pos.len() - 1] - 1;
        if (next_pos + 1) % WIDTH == 0 {
            next_pos += WIDTH;
        }
        p_pos.push(next_pos);
        p_pos.remove(0);
    // Move up
    } else if cur_play == 'w' {
        let mut next_pos = p_pos[p_pos.len() - 1] - WIDTH;
        if next_pos < 0 {
            next_pos = p_pos[p_pos.len() - 1] + WIDTH * (HEIGHT - 1);
        }
        p_pos.push(next_pos);
        p_pos.remove(0);
    // Move down
    } else if cur_play == 's' {
        let mut next_pos = p_pos[p_pos.len() - 1] + WIDTH;
        if next_pos > WIDTH * HEIGHT {
            next_pos = p_pos[p_pos.len() - 1] - WIDTH * (HEIGHT - 1);
        }
        p_pos.push(next_pos);
        p_pos.remove(0);
    }
}

fn is_valid_move(input: char, last_input: char) -> bool {
    !((input == 'd' && last_input == 'a') || (input == 'a' && last_input == 'd') || (input == 'w' && last_input == 's') || (input == 's' && last_input == 'w'))
}

fn spawn_player() -> Vec<i16> {
    let mut rand_lib = rand::thread_rng();
    let initial_pos: i16 = rand_lib.gen_range(0..WIDTH * HEIGHT);
    
    let mut p_positions = Vec::<i16>::new();
    p_positions.push(initial_pos);
    p_positions.push(initial_pos + 1);
    p_positions.push(initial_pos + 2);

    p_positions
}

fn is_game_over(player_coords: &mut Vec<i16>, rocks_coords: &mut Vec<i16>) -> bool {
    let mut game_over = false;

    // Check if the player, the snake, is in the same position as a rock is
    'player_pos: for p_index in 0..player_coords.len() {
        for rock_index in 0..rocks_coords.len() {
            if rocks_coords[rock_index] == player_coords[p_index] {
                game_over = true;
                break 'player_pos;
            }
        }
    }

    game_over
}

fn print_game_over() {
    // See 'https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797'
    println!("\x1b[91m / ___| __ _ _ __ ___   ___   / _ \\__   _____ _ __\\");
    println!("| |  _ / _` | '_ ` _ \\ / _ \\ | | | \\ \\ / / _ \\ '__|");
    println!("| |_| | (_| | | | | | |  __/ | |_| |\\ V /  __/ |");  
    println!(" \\____|\\__,_|_| |_| |_|\\___|  \\___/  \\_/ \\___|_|\x1b[0m");
    println!("Press any key to continue...");
}

fn main() {
    let stdin_channel = spawn_stdin_channel();
    let mut cur_play: char = 'd';
    let mut last_play: char = cur_play;
    let mut p_pos = spawn_player();
    let mut rocks_coords = Vec::<i16>::new();
    let mut game_over: bool = false;
    rocks_coords.push(78);
    rocks_coords.push(80);
    rocks_coords.push(204);

    loop {
        refresh();
        if game_over {
            print_game_over();
        } else {
            print_board(&mut p_pos, &mut rocks_coords);
        }
        
        match stdin_channel.try_recv() {
            Ok(input) => {
                if input != '\0' {
                    // FIXME: Move bug when pressing the same character multiple times
                    if is_valid_move(input, cur_play) && !game_over {
                        last_play = cur_play;
                        cur_play = input;
                    }

                    // Reset game
                    if game_over {
                        game_over = false;
                        p_pos = spawn_player();
                    }
                }
            },
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }

        if !game_over {
            snake_move(cur_play, &mut p_pos);
        }

        if is_game_over(&mut p_pos, &mut rocks_coords) {
            game_over = true;
        }
        sleep(500);
    }
}
