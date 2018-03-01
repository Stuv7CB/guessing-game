use app;
use app::{GameData, AppData, Ids};
use super::conrod;
use conrod::{Colorable, Labelable, Positionable, Sizeable};
use conrod::Widget;
use conrod::widget::text_box;
use conrod::widget::{Canvas, Button, TextBox, Text};

pub fn update(ref mut ui: conrod::UiCell, ids: &Ids, game: &mut GameData, data: &mut AppData) {
    let caption = app::set_caption(&game);

    Canvas::new()
        .color(conrod::color::WHITE)
        .pad(40.0)
        .set(ids.canvas, ui);

    Text::new(caption.as_str())
        .align_top_of(ids.canvas)
        .align_middle_x_of(ids.canvas)
        .center_justify()
        .w_of(ids.canvas)
        .set(ids.title, ui);

    for _click in Button::new()
        .top_left_of(ids.canvas)
        .w_h(100.0, 50.0)
        .label("Guess!")
        .and_if(game.end(), |button|{
            button.label("New game!")
        })
        .set(ids.button, ui)
        {
            if game.end() {
               game.restart()
            }
            else {
                data.info = game.new_guess(&data.guess);
            }
        }

    let attempts = game.get_no_guess().to_string();
    Text::new(attempts.as_str())
        .down_from(ids.button, 10.0)
        .wh_of(ids.button)
        .center_justify()
        .set(ids.count_text, ui);

    for edit in TextBox::new(&data.guess)
        .center_justify()
        .right_from(ids.button, 10.0)
        .w_h(200.0, 50.0)
        .set(ids.textbox, ui)
        {
            match edit {
                text_box::Event::Enter => {
                    data.info = game.new_guess(&data.guess);
                }
                text_box::Event::Update(text) => {
                    data.new_guess(&text);
                }
            }
        }

    Text::new(&data.info)
        .right_from(ids.textbox, 10.0)
        .align_middle_y_of(ids.textbox)
        .set(ids.info_text, ui);
}