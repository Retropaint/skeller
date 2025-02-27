use std::fs;

use egui::{menu, Context, TopBottomPanel, InputState};
use miniquad as mq;
use mq::*;

mod armature_window;
mod bindings;
mod bone_window;
mod mq_backbone;
mod operation_window;
mod top_menu;
mod utils;

use bindings::*;
use mq_backbone::{add_image, Bone, Skelements, Stage, Vec2, Vertex};
use utils::{in_bounding_box, rotate};

impl mq::EventHandler for mq_backbone::Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        self.mq_ctx
            .begin_default_pass(PassAction::clear_color(0., 0., 0.1, 1.));

        self.mq_ctx.apply_pipeline(&self.pipeline);
        self.mq_ctx.apply_bindings(&self.bindings);

        // where the magic happens
        self.egui_mq.run(&mut *self.mq_ctx, |_mq_ctx, egui_ctx| {
            draw_ui(egui_ctx, &mut self.skelements);
        });
        draw_mq(self);

        self.mq_ctx.end_render_pass();
        self.egui_mq.draw(&mut *self.mq_ctx);

        self.mq_ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
        let sk = &mut self.skelements;

        if sk.mouse_pressed_frames < 5 || self.egui_mq.egui_ctx().is_pointer_over_area() {
            return;
        }

        let sens_reduce = 100.;
        let mouse_vel = Vec2 {
            x: (sk.mouse.x - sk.mouse_prev.x) / sens_reduce,
            y: (sk.mouse.y - sk.mouse_prev.y) / sens_reduce,
        };
        if sk.selected_bone == usize::MAX {
            sk.camera.pos.x -= mouse_vel.x;
            sk.camera.pos.y += mouse_vel.y;
        } else {
            let bone = &mut sk.armature.bones[sk.selected_bone];
            match sk.op_mode {
                // translate
                0 => {
                    // counteract parent's rotation
                    // so that it's back to straight up and right
                    let offset = rotate(&mouse_vel, bone.parent_rot);

                    bone.pos.x += offset.x;
                    bone.pos.y -= offset.y;
                }

                // rotate
                1 => bone.rot -= mouse_vel.x,

                // scale
                2 => {
                    bone.scale.x = f32::max(bone.scale.x + mouse_vel.x, 0.);
                    bone.scale.y = f32::max(bone.scale.y - mouse_vel.y, 0.);
                }
                _ => {}
            }
        }
    }

    fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.skelements.mouse_pressed = true;
        self.egui_mq.mouse_button_down_event(mb, x, y);
        let sk = &mut self.skelements;

        // immediately select hovered bone if nothing else is
        if sk.hovered_bone != -1 && sk.selected_bone == usize::MAX {
            sk.selected_bone =
                armature_window::find_bone_idx(&sk.armature.bones, sk.hovered_bone) as usize;
        }
    }

    fn mouse_button_up_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.skelements.mouse_pressed = false;
        self.egui_mq.mouse_button_up_event(mb, x, y);
        let sk = &mut self.skelements;

        // ignore if mouse is on UI or is dragging
        if self.egui_mq.egui_ctx().is_pointer_over_area() || sk.mouse_pressed_frames > 5 {
            return;
        }

        // select hovered bone, or none if nothing's hovered
        if sk.hovered_bone == -1 {
            sk.selected_bone = usize::MAX;
        } else {
            sk.selected_bone =
                armature_window::find_bone_idx(&sk.armature.bones, sk.hovered_bone) as usize;
        }
    }

    fn char_event(&mut self, character: char, _keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.key_down_event(keycode, keymods);
    }

    fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}

/// read temporary files created from file dialogs
fn read_temp_file(skelements: &mut Skelements) {
    if !fs::exists(".skelform_img_path").unwrap() {
        return;
    }

    let fs = fs::read_to_string(".skelform_img_path").unwrap();
    if fs == "" {
        del_temp_files();
        return;
    }

    add_image(fs, &mut skelements.textures);

    skelements.armature.bones[skelements.selected_bone].tex.idx = skelements.textures.len() - 1;

    del_temp_files();
}

fn del_temp_files() {
    #[rustfmt::skip]
    let files = [
        ".skelform_img_path", 
        ".skelform_bone_idx"
    ];
    for f in files {
        if fs::exists(f).unwrap() {
            fs::remove_file(f).unwrap();
        }
    }
}

fn draw_ui(egui_ctx: &Context, skelements: &mut Skelements) {
    egui_ctx.set_zoom_factor(2.);

    read_temp_file(skelements);

    top_menu::draw(&egui_ctx);
    armature_window::draw_armature(egui_ctx, skelements);
    bone_window::draw_bone(egui_ctx, skelements);
    operation_window::draw(egui_ctx, skelements);

    egui_ctx.input(|i| {
        if let Some(m) = i.pointer.hover_pos() {
            // get window size
            skelements.window_size = Vec2 {
                x: i.screen_rect().size().x,
                y: i.screen_rect().size().y,
            };

            // get mouse data
            skelements.mouse_prev = skelements.mouse.clone();
            skelements.mouse = Vec2 { x: m.x, y: m.y };
        }

        println!("{}", i.raw_scroll_delta);
    });

    if skelements.mouse_pressed {
        skelements.mouse_pressed_frames += 1;
    } else {
        skelements.mouse_pressed_frames = 0;
    }

    // debug stuff
    if !skelements.made_test {
        armature_window::create_bone(&mut skelements.armature.bones);
        armature_window::create_bone(&mut skelements.armature.bones);
        armature_window::create_bone(&mut skelements.armature.bones);
        skelements.armature.bones[1].parent_id = skelements.armature.bones[0].id;
        skelements.armature.bones[2].parent_id = skelements.armature.bones[1].id;
        add_image(
            "/Users/o/downloads/ferris.png".to_string(),
            &mut skelements.textures,
        );
        skelements.armature.bones[0].pos.x += 0.25;
        skelements.armature.bones[1].pos.x += 0.25;
        skelements.armature.bones[2].pos.x += 0.25;
        skelements.armature.bones[0].tex.idx = usize::MAX;
        skelements.armature.bones[1].tex.idx = 0;
        skelements.armature.bones[2].tex.idx = 0;
        skelements.selected_bone = 2;
        skelements.made_test = true;
    }
}

fn draw_mq(stage: &mut Stage) {
    let sk = &mut stage.skelements;

    /*
        many visual effects should not affect the actual
        properties of bones, so it's best to separate
        the rendering pipeline with a copy of them
        and let us go wild with manipulation
    */
    let mut temp_bones: Vec<Bone> = vec![];

    let mut verts: Vec<Vec<Vertex>> = vec![];

    let mut i = 0;
    for b in &mut sk.armature.bones {
        if sk.textures.len() == 0 {
            continue;
        }
        temp_bones.push(b.clone());

        // get parent so it can be inherited
        let mut p = Bone {
            scale: Vec2 { x: 1., y: 1. },
            ..Default::default()
        };
        if let Some(pp) = armature_window::find_bone(&temp_bones, b.parent_id) {
            p = pp.clone();
        }
        let tb = &mut temp_bones[i];

        // inherit said parent
        tb.scale.x *= p.scale.x;
        tb.scale.y *= p.scale.y;
        tb.rot += p.rot;
        b.parent_rot = p.rot;

        // adjust position based on parent's scale
        tb.pos.x *= p.scale.x;
        tb.pos.y *= p.scale.y;

        // rotate based on parent
        tb.pos = rotate(&tb.pos, p.rot);

        // move with parent
        //
        // this has to be last, as it's easiest for all
        // inheritance logic above to process at origin (0, 0)
        tb.pos.x += p.pos.x;
        tb.pos.y += p.pos.y;

        // external offsets (camera, window, etc)
        if tb.parent_id == -1 {
            tb.pos.x -= sk.camera.pos.x;
            tb.pos.y -= sk.camera.pos.y;

            // adjust bone scale based on window aspect ratio
            let ratio = f32::max(sk.window_size.x, sk.window_size.y)
                / f32::min(sk.window_size.x, sk.window_size.y);
            if sk.window_size.x > sk.window_size.y {
                tb.scale.x /= ratio;
            } else {
                tb.scale.y /= ratio;
            }

            // zoom
            tb.pos.x *= sk.camera.zoom;
            tb.pos.y *= sk.camera.zoom;
            tb.scale.x *= sk.camera.zoom;
            tb.scale.y *= sk.camera.zoom;
        }

        // provide vertices, for use later
        let mut size = &Vec2::default();
        if tb.tex.idx != usize::MAX {
            size = &sk.textures[b.tex.idx].size;
        }
        let v = rect_tex_verts(&tb.pos, &tb.scale, &size, tb.rot);
        verts.push(v);

        i += 1;
    }

    if temp_bones.len() == 0 {
        return;
    }

    // lowest bones in hierarchy should be targeted first,
    // which effectively means reversing the list
    temp_bones.reverse();
    let len = temp_bones.len();
    i = len - 1;

    // get bone that's being hovered on
    sk.hovered_bone = -1;
    for tb in &mut temp_bones {
        if sk.hovered_bone == -1
            && sk.mouse_pressed_frames < 5
            && !stage.egui_mq.egui_ctx().is_pointer_over_area()
            && in_bounding_box(&sk.mouse, &verts[i], &sk.window_size)
        {
            sk.hovered_bone = tb.id;
        }

        if i != 0 {
            i -= 1;
        }
    }

    // go back to normal order for rendering
    temp_bones.reverse();

    i = 0;
    for tb in temp_bones {
        // render appropriate effect if this is the hovered bone
        // and it's not already selected
        if sk.hovered_bone == tb.id && sk.selected_bone != i {
            let b = rect_bind(
                &mut stage.mq_ctx,
                &verts[i],
                &Vec2 {
                    x: &sk.textures[tb.tex.idx].size.x * tb.scale.x,
                    y: &sk.textures[tb.tex.idx].size.y * tb.scale.y,
                },
                [255, 255, 255, 100],
            );
            stage.mq_ctx.apply_bindings(&b);
            stage.mq_ctx.draw(0, 12, 1);
        }

        // the fun part
        if tb.tex.idx != usize::MAX {
            let (bindings, _) = rect_tex(
                &mut stage.mq_ctx,
                &tb.pos,
                &tb.scale,
                &sk.textures[tb.tex.idx],
                tb.rot,
            );
            stage.mq_ctx.apply_bindings(&bindings);
            stage.mq_ctx.draw(0, 12, 1);
        }

        // render helper arrows if this is selected
        if sk.selected_bone == i {
            // up arrow
            draw_helper_arrows(
                &mut stage.mq_ctx,
                &tb.pos,
                &Vec2 { x: 0., y: 0.5 },
                &tb,
                &sk.window_size,
                &sk.mouse,
                0.,
            );

            // right arrow
            draw_helper_arrows(
                &mut stage.mq_ctx,
                &tb.pos,
                &Vec2 { x: 0., y: 0.5 },
                &tb,
                &sk.window_size,
                &sk.mouse,
                (-90. as f32).to_radians(),
            );
        }

        i += 1;
    }
}

fn draw_helper_arrows(
    mut mq_ctx: &mut Box<dyn RenderingBackend>,
    pos: &Vec2,
    offset: &Vec2,
    bone: &Bone,
    window_size: &Vec2,
    mouse: &Vec2,
    rot: f32,
) {
    #[rustfmt::skip]
    macro_rules! tri_bind {($color:expr) => {
        tri_bind(
            &mut mq_ctx,
            pos.clone(),
            offset.clone(),
            Vec2{x: 0.1, y: 0.1},
            bone.rot + rot,
            $color,
        )
    };}

    let (mut tri, verts) = tri_bind!([100, 100, 100, 0]);

    // recolor triangle if it's hovered
    if in_bounding_box(&mouse, &verts, &window_size) {
        (tri, _) = tri_bind!([60, 60, 60, 0]);
    }

    mq_ctx.apply_bindings(&tri);
    mq_ctx.draw(0, 12, 1);
}

fn main() {
    let conf = mq::conf::Conf {
        high_dpi: true,
        window_width: 600,
        window_height: 600,
        ..Default::default()
    };

    mq::start(conf, || Box::new(mq_backbone::Stage::new(vec![])));
}
