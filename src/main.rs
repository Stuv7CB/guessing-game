#[macro_use]
extern crate conrod;
use conrod::backend::glium::glium::{self, Surface};

extern crate find_folder;

extern crate rand;

mod app;
use app::{GameData, AppData, Ids};

mod logic;

fn main() {
    let data = AppData::new(450, 350, "Guessing Game");

    let game = GameData::new(10, [1, 50]);

    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title(data.title.clone())
        .with_dimensions(data.width, data.height);

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    let (event_tx, event_rx) = std::sync::mpsc::channel();

    let (render_tx, render_rx) = std::sync::mpsc::channel();

    let events_loop_proxy = events_loop.create_proxy();

    std::thread::spawn(move || run_conrod(event_rx, render_tx, events_loop_proxy,data, game));

    let mut last_update = std::time::Instant::now();
    let mut closed = false;
    while !closed {
        let sixteen_ms = std::time::Duration::from_millis(16);
        let now = std::time::Instant::now();
        let duration_since_last_update = now.duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        events_loop.run_forever(|event| {
            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                event_tx.send(event).unwrap();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::Closed |
                    glium::glutin::WindowEvent::KeyboardInput {
                        input: glium::glutin::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => {
                        closed = true;
                        return glium::glutin::ControlFlow::Break;
                    },
                    // We must re-draw on `Resized`, as the event loops become blocked during
                    // resize on macOS.
                    glium::glutin::WindowEvent::Resized(..) => {
                        if let Some(primitives) = render_rx.iter().next() {
                            draw(&display, &mut renderer, &image_map, &primitives);
                        }
                    },
                    _ => {},
                },
                glium::glutin::Event::Awakened => return glium::glutin::ControlFlow::Break,
                _ => (),
            }

            glium::glutin::ControlFlow::Continue
        });

        // Draw the most recently received `conrod::render::Primitives` sent from the `Ui`.
        if let Some(primitives) = render_rx.try_iter().last() {
            draw(&display, &mut renderer, &image_map, &primitives);
        }

        last_update = std::time::Instant::now();
    }
}

fn run_conrod(event_rx: std::sync::mpsc::Receiver<conrod::event::Input>,
              render_tx: std::sync::mpsc::Sender<conrod::render::OwnedPrimitives>,
              events_loop_proxy: glium::glutin::EventsLoopProxy,
              mut data: AppData,
              mut game: GameData)
{
    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([data.width as f64, data.height as f64]).build();

    // Generate the widget identifiers.
    let ids = Ids::new(ui.widget_id_generator());

    let font_path = app::load_font("NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Many widgets require another frame to finish drawing after clicks or hovers, so we
    // insert an update into the conrod loop using this `bool` after each event.
    let mut needs_update = true;

    'conrod: loop {
        // Collect any pending events.
        let mut events = Vec::new();
        while let Ok(event) = event_rx.try_recv() {
            events.push(event);
        }

        // If there are no events pending, wait for them.
        if events.is_empty() || !needs_update {
            match event_rx.recv() {
                Ok(event) => events.push(event),
                Err(_) => break 'conrod,
            };
        }

        needs_update = false;

        for event in events {
            ui.handle_event(event);
            needs_update = true;
        }

        logic::update(ui.set_widgets(), &ids, &mut game, &mut data);

        if let Some(primitives) = ui.draw_if_changed() {
            if render_tx.send(primitives.owned()).is_err()
                || events_loop_proxy.wakeup().is_err() {
                break 'conrod;
            }
        }
    }
}

fn draw(display: &glium::Display,
        renderer: &mut conrod::backend::glium::Renderer,
        image_map: &conrod::image::Map<glium::Texture2d>,
        primitives: &conrod::render::OwnedPrimitives)
{
    renderer.fill(display, primitives.walk(), &image_map);
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 1.0);
    renderer.draw(display, &mut target, &image_map).unwrap();
    target.finish().unwrap();
}