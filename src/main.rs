#![allow(dead_code)]

extern crate rg3d_core;
extern crate rg3d;
extern crate rand;
extern crate rg3d_physics;

mod level;
mod player;
mod weapon;
mod bot;
mod projectile;
mod menu;

use std::{
    fs::File,
    path::Path,
    time::Instant,
    io::Write,
    time,
    thread,
    time::Duration,
    collections::VecDeque,
};
use rg3d::{
    engine::{
        Engine,
        EngineInterfaceMut,
    },
    gui::{
        node::{UINode, UINodeKind},
        text::TextBuilder,
    },
    scene::particle_system::CustomEmitterFactory,
    WindowEvent,
    ElementState,
    VirtualKeyCode,
    Event,
    EventsLoop,
};
use crate::level::{Level, CylinderEmitter};
use rg3d_core::{
    pool::Handle,
    visitor::{
        Visitor,
        VisitResult,
        Visit,
    },
};
use rg3d_sound::{
    buffer::BufferKind,
    source::{Source, SourceKind},
};
use crate::menu::Menu;
use rg3d::gui::event::{UIEvent, UIEventKind};


pub struct Game {
    menu: Menu,
    events_loop: EventsLoop,
    engine: Engine,
    level: Option<Level>,
    debug_text: Handle<UINode>,
    debug_string: String,
    running: bool,
    last_tick_time: time::Instant,
}

pub struct GameTime {
    elapsed: f64,
    delta: f32,
}

impl Game {
    pub fn new() -> Game {
        let events_loop = EventsLoop::new();

        let primary_monitor = events_loop.get_primary_monitor();
        let mut monitor_dimensions = primary_monitor.get_dimensions();
        monitor_dimensions.height *= 0.7;
        monitor_dimensions.width *= 0.7;
        let window_size = monitor_dimensions.to_logical(primary_monitor.get_hidpi_factor());

        let window_builder = rg3d::WindowBuilder::new()
            .with_title("Rusty Shooter")
            .with_dimensions(window_size)
            .with_resizable(true);

        let mut engine = Engine::new(window_builder, &events_loop).unwrap();

        if let Ok(mut factory) = CustomEmitterFactory::get() {
            factory.set_callback(Box::new(|kind| {
                match kind {
                    0 => Ok(Box::new(CylinderEmitter::new())),
                    _ => Err(String::from("invalid custom emitter kind"))
                }
            }))
        }

        let EngineInterfaceMut { sound_context, resource_manager, .. } = engine.interface_mut();

        let buffer = resource_manager.request_sound_buffer(Path::new("data/sounds/Sonic_Mayhem_Collapse.wav"), BufferKind::Stream).unwrap();
        let mut source = Source::new(SourceKind::Flat, buffer).unwrap();
        source.play();
        source.set_gain(0.25);
        sound_context.lock().unwrap().add_source(source);

        let mut game = Game {
            running: true,
            events_loop,
            menu: Menu::new(&mut engine),
            debug_text: Handle::NONE,
            engine,
            level: None,
            debug_string: String::new(),
            last_tick_time: time::Instant::now(),
        };
        game.create_ui();
        game
    }

    pub fn create_ui(&mut self) {
        let EngineInterfaceMut { ui, .. } = self.engine.interface_mut();

        self.debug_text = TextBuilder::new()
            .with_width(400.0)
            .with_height(200.0)
            .build(ui);
    }

    pub fn save_game(&mut self) -> VisitResult {
        let mut visitor = Visitor::new();

        // Visit engine state first.
        self.engine.visit("Engine", &mut visitor)?;

        self.level.visit("Level", &mut visitor)?;

        // Debug output
        if let Ok(mut file) = File::create(Path::new("save.txt")) {
            file.write_all(visitor.save_text().as_bytes()).unwrap();
        }

        visitor.save_binary(Path::new("save.bin"))
    }

    pub fn load_game(&mut self) {
        match Visitor::load_binary(Path::new("save.bin")) {
            Ok(mut visitor) => {
                // Clean up.
                self.destroy_level();

                // Load engine state first
                match self.engine.visit("Engine", &mut visitor) {
                    Ok(_) => {
                        println!("Engine state successfully loaded!");

                        // Then load game state.
                        match self.level.visit("Level", &mut visitor) {
                            Ok(_) => {
                                println!("Game state successfully loaded!");

                                // Hide menu only of we successfully loaded a save.
                                self.set_menu_visible(false)
                            }
                            Err(e) => println!("Failed to load game state! Reason: {}", e)
                        }
                    }
                    Err(e) => println!("Failed to load engine state! Reason: {}", e)
                }
            }
            Err(e) => {
                println!("failed to load a save, reason: {}", e);
            }
        }
    }

    fn destroy_level(&mut self) {
        if let Some(ref mut level) = self.level.take() {
            level.destroy(&mut self.engine);
        }
    }

    pub fn start_new_game(&mut self) {
        self.destroy_level();
        self.level = Some(Level::new(&mut self.engine));
        self.set_menu_visible(false);
    }

    pub fn process_ui_event(&mut self, event: &mut UIEvent) {
        match event.kind {
            UIEventKind::Click => {
                if event.source() == self.menu.btn_new_game {
                    self.start_new_game();
                    event.handled = true;
                } else if event.source() == self.menu.btn_save_game {
                    match self.save_game() {
                        Ok(_) => println!("successfully saved"),
                        Err(e) => println!("failed to make a save, reason: {}", e),
                    }
                    event.handled = true;
                } else if event.source() == self.menu.btn_load_game {
                    self.load_game();
                    event.handled = true;
                } else if event.source() == self.menu.btn_quit_game {
                    self.destroy_level();
                    self.running = false;
                    event.handled = true;
                }
            }
            _ => ()
        }
    }

    pub fn set_menu_visible(&mut self, visible: bool) {
        self.menu.set_visible(&mut self.engine, visible)
    }

    pub fn is_menu_visible(&self) -> bool {
        self.menu.is_visible(&self.engine)
    }

    pub fn update(&mut self, time: &GameTime) {
        if let Some(ref mut level) = self.level {
            level.update(&mut self.engine, time);
        }
        self.engine.update(time.delta);
    }

    pub fn update_statistics(&mut self, elapsed: f64) {
        let EngineInterfaceMut { ui, renderer, .. } = self.engine.interface_mut();

        self.debug_string.clear();
        use std::fmt::Write;
        let statistics = renderer.get_statistics();
        write!(self.debug_string,
               "Pure frame time: {:.2} ms\n\
               Capped frame time: {:.2} ms\n\
               FPS: {}\n\
               Potential FPS: {}\n\
               Up time: {:.2} s",
               statistics.pure_frame_time * 1000.0,
               statistics.capped_frame_time * 1000.0,
               statistics.frames_per_second,
               statistics.potential_frame_per_second,
               elapsed
        ).unwrap();

        if let Some(ui_node) = ui.get_node_mut(self.debug_text) {
            if let UINodeKind::Text(text) = ui_node.get_kind_mut() {
                text.set_text(self.debug_string.as_str());
            }
        }
    }

    pub fn limit_fps(&mut self, value: f64) {
        let current_time = time::Instant::now();
        let render_call_duration = current_time.duration_since(self.last_tick_time).as_secs_f64();
        self.last_tick_time = current_time;
        let desired_frame_time = 1.0 / value;
        if render_call_duration < desired_frame_time {
            thread::sleep(Duration::from_secs_f64(desired_frame_time - render_call_duration));
        }
    }

    fn process_dispatched_event(&mut self, event: &WindowEvent) {
        let EngineInterfaceMut { ui, .. } = self.engine.interface_mut();

        // Some events can be consumed so they won't be dispatched further,
        // this allows to catch events by UI for example and don't send them
        // to player controller so when you click on some button in UI you
        // won't shoot from your current weapon in game.
        let event_processed = ui.process_input_event(event);

        if !event_processed {
            if let Some(ref mut level) = self.level {
                if let Some(player) = level.get_player_mut() {
                    player.process_event(event);
                }
            }
        }
    }

    pub fn process_input_event(&mut self, event: Event) {
        if let Event::WindowEvent { event, .. } = event {
            self.process_dispatched_event(&event);

            // Some events processed in any case.
            match event {
                WindowEvent::CloseRequested => self.running = false,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let ElementState::Pressed = input.state {
                        if let Some(key) = input.virtual_keycode {
                            if key == VirtualKeyCode::Escape {
                                self.set_menu_visible(!self.is_menu_visible());
                            }
                        }
                    }
                }
                _ => ()
            }

            self.menu.process_input_event(&mut self.engine, &event);
        }
    }

    pub fn run(&mut self) {
        let fixed_fps = 60.0;
        let fixed_timestep = 1.0 / fixed_fps;
        let clock = Instant::now();
        let mut game_time = GameTime {
            elapsed: 0.0,
            delta: fixed_timestep,
        };

        let mut events = VecDeque::new();
        while self.running {
            let mut dt = clock.elapsed().as_secs_f64() - game_time.elapsed;
            while dt >= fixed_timestep as f64 {
                dt -= fixed_timestep as f64;
                game_time.elapsed += fixed_timestep as f64;

                self.events_loop.poll_events(|event| {
                    events.push_back(event);
                });

                while let Some(event) = events.pop_front() {
                    self.process_input_event(event);
                }

                while let Some(mut ui_event) = self.engine.get_ui_mut().poll_ui_event() {
                    self.menu.process_ui_event(&mut self.engine, &mut ui_event);
                    self.process_ui_event(&mut ui_event);
                }

                self.update(&game_time);
            }

            self.update_statistics(game_time.elapsed);

            // Render at max speed
            self.engine.render().unwrap();

            // Make sure to cap update rate to 60 FPS.
            self.limit_fps(fixed_fps as f64);
        }
        self.destroy_level();
    }
}

fn main() {
    Game::new().run();
}