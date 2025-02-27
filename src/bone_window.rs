use std::f32::consts::PI;
use std::io::Write;
use std::rc::Rc;
use std::{fs::File, thread};

use egui::{Align2, Context, Layout, Ui, Vec2};

use crate::mq_backbone::{Bone, Skelements};

pub fn draw_bone(egui_ctx: &Context, skelements: &mut Skelements) {
    egui::Window::new("Bone")
        .movable(false)
        .anchor(Align2::RIGHT_TOP, Vec2 { x: -20., y: 35. })
        .max_width(150.)
        .show(egui_ctx, |ui| {
            let mut bone: Bone = Bone::default();
            if skelements.selected_bone != usize::MAX {
                bone = skelements.armature.bones[skelements.selected_bone].clone();
            } else {
                ui.disable();
            }
            ui.horizontal(|ui| {
                let l = ui.label("Name:");
                ui.text_edit_singleline(&mut bone.name).labelled_by(l.id);
            });
            ui.horizontal(|ui| {
                ui.label("Texture:");
                let bone_idx = skelements.selected_bone;
                if ui.button("Get Image").clicked() {
                    open_file_dialog(bone_idx);
                };
            });
            if skelements.selected_bone == usize::MAX {
                return;
            }
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.label("x:");
                float_input(
                    ui,
                    &mut skelements.armature.bones[skelements.selected_bone].pos.x,
                );
                ui.label("y:");
                float_input(
                    ui,
                    &mut skelements.armature.bones[skelements.selected_bone].pos.y,
                );
            });
            ui.horizontal(|ui| {
                ui.label("Scale:");
                ui.label("x:");
                float_input(
                    ui,
                    &mut skelements.armature.bones[skelements.selected_bone].scale.x,
                );
                ui.label("y:");
                float_input(
                    ui,
                    &mut skelements.armature.bones[skelements.selected_bone].scale.y,
                );
            });
            ui.horizontal(|ui| {
                ui.label("Rotation:");
                let deg = skelements.armature.bones[skelements.selected_bone].rot / PI * 180.;
                let mut str = deg.round().to_string();
                if !str.contains(".") {
                    str.push('.');
                }
                ui.add_sized([30., 20.], egui::TextEdit::singleline(&mut str));
                if let Ok(f) = str.parse::<f32>() {
                    skelements.armature.bones[skelements.selected_bone].rot = f * PI / 180.;
                } else {
                    skelements.armature.bones[skelements.selected_bone].rot = 0.;
                }
            });
            if ui.button("Delete Bone").clicked() {
                skelements.armature.bones.remove(skelements.selected_bone);
                skelements.selected_bone = usize::MAX;
            };
        });
}

fn open_file_dialog(bone_idx: usize) {
    thread::spawn(move || {
        let task = rfd::FileDialog::new().pick_file();
        let mut img_path = File::create(".skelform_img_path").unwrap();
        img_path
            .write_all(task.unwrap().as_path().to_str().unwrap().as_bytes())
            .unwrap();
        let mut bone_idx_file = File::create(".skelform_bone_idx").unwrap();
        bone_idx_file
            .write_all(bone_idx.to_string().as_bytes())
            .unwrap();
    });
}

// helper for editable float inputs
fn float_input(ui: &mut Ui, float: &mut f32) {
    let mut str = float.to_string();
    if !str.contains(".") {
        str.push('.');
    }
    ui.add_sized([30., 20.], egui::TextEdit::singleline(&mut str));
    if let Ok(f) = str.parse::<f32>() {
        *float = f;
    } else {
        *float = 0.;
    }
}
