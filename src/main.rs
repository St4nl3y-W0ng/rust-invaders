use std::{error::Error, io::{self, stdout}, time::{Duration, Instant}, thread};

use crossterm::{terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand, cursor::{Show, Hide}, event::{self, Event, KeyCode}};
use crossbeam::channel::{self};
use invaders::{render::render, frame::{self, new_frame, Drawable}, player::Player};
use rusty_audio::Audio;

fn main() -> Result <(), Box<dyn Error>>{
    // Audio
    let mut audio = Audio::new();
    audio.add("explode", "explode.wav");
    audio.add("lose", "lose.wav");
    audio.add("move", "move.wav");
    audio.add("pew", "pew.wav");
    audio.add("startup", "startup.wav");
    audio.add("win", "win.wav");
    audio.play("win");
    // Render loop in a separate thread
    let (render_tx, render_rx) = channel::unbounded();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = stdout();
        render(&mut stdout, &last_frame, &last_frame, true);
        loop{
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Game Loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    'gameloop: loop{
        // Per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        // Input
        while event::poll(Duration::default())?{
            if let Event::Key(key_event) = event::read()?{
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot(){
                            audio.play("pew");
                        }
                    },
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    },
                    _ => {}
                }
            }
        }
        
        // Updates
        player.update(delta);

        // Draw and render
        player.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame); // catches and ignores err  
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    println!("Hello");
    
    Ok(())
}
