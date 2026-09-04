use egui::{Margin, Response, TextEdit, TextStyle, Ui};

/// A readable recipe text area that grows with its content. The surrounding
/// form handles scrolling; Tab remains available for moving between fields.
#[derive(Default)]
pub struct Notebook;

impl Notebook {
    pub fn ui(
        ui: &mut Ui,
        value: &mut String,
        id_source: impl egui::AsIdSalt,
        rows: usize,
        hint: &str,
    ) -> Response {
        ui.scope(|ui| {
            ui.spacing_mut().extra_text_line_spacing = 7.0;
            let mut font = TextStyle::Body.resolve(ui.style());
            font.size = font.size.max(16.0);
            ui.add(
                TextEdit::multiline(value)
                    .id_salt(id_source)
                    .desired_width(f32::INFINITY)
                    .desired_rows(rows)
                    .margin(Margin::symmetric(8, 10))
                    .font(font.clone())
                    .hint_text(egui::RichText::new(hint).font(font))
                    .lock_focus(false),
            )
        })
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_moves_between_recipe_fields_without_changing_their_text() {
        let ctx = egui::Context::default();
        crate::set_theme(&ctx, crate::Theme::default());
        let mut ingredients = "1 tablespoon olive oil\n1 onion".to_owned();
        let mut instructions = "Chop the onion.".to_owned();

        let mut render = |input| {
            let mut fields = None;
            let mut output = ctx.run_ui(input, |ui| {
                fields = Some((
                    Notebook::ui(ui, &mut ingredients, "ingredients", 6, ""),
                    Notebook::ui(ui, &mut instructions, "instructions", 5, ""),
                ));
            });
            output.textures_delta.clear();
            fields.unwrap()
        };

        let (ingredients_field, instructions_field) = render(Default::default());
        ingredients_field.request_focus();
        for (modifiers, expected_focus) in [
            (egui::Modifiers::NONE, instructions_field.id),
            (egui::Modifiers::SHIFT, ingredients_field.id),
        ] {
            render(egui::RawInput {
                events: vec![
                    egui::Event::ModifiersChanged(modifiers),
                    egui::Event::Key {
                        key: egui::Key::Tab,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers,
                    },
                ],
                ..Default::default()
            });
            // egui applies backward traversal on the following pass.
            render(egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers,
                }],
                ..Default::default()
            });
            assert_eq!(ctx.memory(|memory| memory.focused()), Some(expected_focus));
        }
        assert_eq!(ingredients, "1 tablespoon olive oil\n1 onion");
        assert_eq!(instructions, "Chop the onion.");
    }
}
