use bevy::prelude::*;

pub fn point_rect_collision(
    point: Vec2,
    rect_center: Vec2,
    rect_half_size: Vec2,
    point_radius: f32,
) -> bool {
    let dist_x = (point.x - rect_center.x).abs();
    let dist_y = (point.y - rect_center.y).abs();
    dist_x < (rect_half_size.x + point_radius) && dist_y < (rect_half_size.y + point_radius)
}

#[inline]
pub fn in_range(a: Vec2, b: Vec2, range: f32) -> bool {
    a.distance_squared(b) < range * range
}

pub fn clamp_to_bounds(pos: Vec2, bounds: f32) -> Vec2 {
    Vec2::new(pos.x.clamp(-bounds, bounds), pos.y.clamp(-bounds, bounds))
}

pub fn direction_to(from: Vec2, to: Vec2) -> Vec2 {
    (to - from).normalize_or_zero()
}
