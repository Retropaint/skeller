use egui::*;

use crate::mq_backbone::{Bone, BoneTexture, Skelements, Vec2};

pub fn draw_armature(egui_ctx: &Context, skelements: &mut Skelements) {
    let bones = &mut skelements.armature.bones;

    egui::Window::new("Armature")
        .movable(false)
        .show(egui_ctx, |ui| {
            // bone options
            ui.horizontal(|ui| {
                if ui.button("New Bone").clicked() {
                    create_bone(bones);
                }
                let drag_name = if skelements.dragging { "Stay" } else { "Drag" };
                if ui.button(drag_name).clicked() {
                    skelements.dragging = !skelements.dragging;
                }
            });

            if bones.len() == 0 {
                return;
            }

            // hierarchy
            let frame = Frame::default().inner_margin(10.0);
            ui.dnd_drop_zone::<i32, _>(frame, |ui| {
                let mut idx = 0;
                for s in bones.clone() {
                    ui.horizontal(|ui| {
                        // add space to the left if this is a child
                        let mut nb = &s;
                        while nb.parent_id != -1 {
                            nb = find_bone(&bones, nb.parent_id).unwrap();
                            ui.add_space(20.);
                        }

                        /*
                            draggable buttons in egui don't seem well-supported, because
                            dnd_drag_source seems to physically block it. When hovering on
                            the edge of a button inside one, it will be clickable but not
                            draggable
                        */

                        if skelements.dragging {
                            // add draggable labels
                            let id = Id::new(("bone", idx, 0));
                            let d = ui
                                .dnd_drag_source(id, idx, |ui| {
                                    ui.label(RichText::new(&s.name.to_string()));
                                })
                                .response;
                            check_bone_dragging(bones, ui, d, idx);
                        } else {
                            // regular, boring buttons

                            let mut col = Color32::from_rgb(60, 60, 60);
                            if idx as usize == skelements.selected_bone {
                                col = Color32::from_rgb(100, 100, 100);
                            }

                            if ui.add(Button::new(&s.name.to_string()).fill(col)).clicked() {
                                skelements.selected_bone = idx as usize;
                            };
                        }

                        idx += 1;
                    });
                }
            });
        });
}

pub fn create_bone(bones: &mut Vec<Bone>) {
    bones.push(Bone {
        name: "bone".to_string() + &bones.len().to_string(),
        parent_id: -1,
        id: generate_id(&bones),
        tex: BoneTexture {
            idx: usize::MAX,
            ..BoneTexture::default()
        },
        scale: Vec2 { x: 1., y: 1. },
        ..Default::default()
    });
}

fn check_bone_dragging(bones: &mut Vec<Bone>, ui: &mut Ui, drag: Response, idx: i32) {
    if let (Some(pointer), Some(hovered_payload)) = (
        ui.input(|i| i.pointer.interact_pos()),
        drag.dnd_hover_payload::<i32>(),
    ) {
        let rect = drag.rect;

        let stroke = egui::Stroke::new(1.0, Color32::WHITE);
        let move_type = if *hovered_payload == idx {
            // not moved
            -1
        } else if pointer.y < rect.center().y {
            // above bone (move dragged bone above it)
            ui.painter().hline(rect.x_range(), rect.top(), stroke);
            1
        } else {
            // in bone (turn dragged bone to child)
            ui.painter().hline(rect.x_range(), rect.top(), stroke);
            ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
            ui.painter().vline(rect.right(), rect.y_range(), stroke);
            ui.painter().vline(rect.left(), rect.y_range(), stroke);
            0
        };

        if let Some(dragged_payload) = drag.dnd_release_payload::<i32>() {
            // ignore if target bone is a child of this
            let mut children: Vec<Bone> = vec![];
            get_all_children(
                bones,
                &mut children,
                bones[*dragged_payload as usize].clone(),
            );
            for c in children {
                if bones[idx as usize].id == c.id {
                    return;
                }
            }

            if move_type == 0 {
                // move dragged bone above target
                bones[*dragged_payload as usize].parent_id = bones[idx as usize].clone().id;
                move_bone(bones, *dragged_payload, idx, true);
            } else if move_type == 1 {
                // set dragged bone's parent as target
                bones[*dragged_payload as usize].parent_id = bones[idx as usize].clone().parent_id;
                move_bone(bones, *dragged_payload, idx, false);
            }
        }
    }
}

pub fn move_bone(bones: &mut Vec<Bone>, old_idx: i32, new_idx: i32, is_setting_parent: bool) {
    let main = bones[old_idx as usize].clone();
    let anchor = bones[new_idx as usize].clone();

    // gather all bones to be moved (this and its children)
    let mut to_move: Vec<Bone> = vec![main.clone()];
    get_all_children(bones, &mut to_move, main);

    // remove them
    for _ in &to_move {
        bones.remove(old_idx as usize);
    }

    // re-add them in the new positions
    if is_setting_parent {
        to_move.reverse();
    }
    for b in &to_move {
        bones.insert(
            find_bone_idx(bones, anchor.id) as usize + is_setting_parent as usize,
            b.clone(),
        );
    }
}

/// Retrieve all children of this bone (recursive)
pub fn get_all_children(bones: &Vec<Bone>, children_vec: &mut Vec<Bone>, parent: Bone) {
    let mut i: usize = 1;
    let idx = find_bone_idx(bones, parent.id);

    #[rustfmt::skip]
    macro_rules! check_bounds {() => {
        if idx as usize + i > bones.len() - 1 {
            return;
        }};
    }

    check_bounds!();

    while bones[idx as usize + i].parent_id == parent.id {
        children_vec.push(bones[idx as usize + i].clone());
        get_all_children(bones, children_vec, bones[idx as usize + i].clone());
        i += 1;
        check_bounds!();
    }
}

pub fn find_bone(bones: &Vec<Bone>, id: i32) -> Option<&Bone> {
    for b in bones {
        if b.id == id {
            return Some(&b);
        }
    }
    None
}

pub fn find_bone_idx(bones: &Vec<Bone>, id: i32) -> i32 {
    let mut i = 0;
    for b in bones {
        if b.id == id {
            return i;
        }
        i += 1;
    }
    -1
}

// generate non-clashing id
pub fn generate_id(bones: &Vec<Bone>) -> i32 {
    let mut idx = 0;
    while idx == does_id_exist(bones, idx) {
        idx += 1;
    }
    return idx;
}

fn does_id_exist(bones: &Vec<Bone>, id: i32) -> i32 {
    for b in bones {
        if b.id == id {
            return id;
        }
    }
    return -1;
}
