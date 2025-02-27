use egui::{Button, Color32, Context};

use crate::mq_backbone::Skelements;

pub fn draw(egui_ctx: &Context, skelements: &mut Skelements) {
    egui::Window::new("Operations")
        .movable(false)
        .show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                #[rustfmt::skip]
            let buttons = [
                "Translate", 
                "Rotate", 
                "Scale"
            ];

                let mut i = 0;
                for b in buttons {
                    let mut col = Color32::from_rgb(30, 30, 30);
                    if skelements.op_mode == i {
                        col = Color32::from_rgb(60, 60, 60);
                    }
                    if ui.add(Button::new(b).fill(col)).clicked() {
                        skelements.op_mode = i;
                    }
                    i += 1;
                }
            });
        });
}
