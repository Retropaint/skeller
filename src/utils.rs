use crate::{mq_backbone::Vertex, Vec2};

/// rotate a point via rotation matrix
pub fn rotate(point: &Vec2, rot: f32) -> Vec2 {
    Vec2 {
        x: point.x * rot.cos() - point.y * rot.sin(),
        y: point.x * rot.sin() + point.y * rot.cos(),
    }
}

pub fn in_bounding_box(point: &Vec2, verts: &Vec<Vertex>, window_size: &Vec2) -> bool {
    // get the bound based on infinitely-long lines
    let mut top = -f32::INFINITY;
    let mut bot = f32::INFINITY;
    let mut left = f32::INFINITY;
    let mut right = -f32::INFINITY;
    for v in verts {
        left = f32::min(left, v.pos.x);
        right = f32::max(right, v.pos.x);
        bot = f32::min(bot, v.pos.y);
        top = f32::max(top, v.pos.y);
    }

    // convert bound positions to screen space
    let half = Vec2 {
        x: window_size.x / 2.,
        y: window_size.y / 2.,
    };
    top = half.y - (half.y * top);
    bot = half.y - (half.y * bot);
    left = half.x + (half.x * left);
    right = half.x + (half.x * right);

    // finally, check if point is inside
    point.y > top && point.y < bot && point.x > left && point.x < right
}
