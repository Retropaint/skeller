use crate::{menu, Context, TopBottomPanel};

pub fn draw(ctx: &Context) {
    TopBottomPanel::top("test").show(ctx, |ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    println!("Open clicked!");
                }
                if ui.button("Save").clicked() {
                    println!("Save clicked!");
                }
            });
        });
    });
}
