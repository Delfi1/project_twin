use bevy::prelude::*;

// Контроллер камеры, ввода пользователя
#[derive(Component)]
pub struct Controller;

pub const SPEED: f32 = 200.0;
pub const ZOOM_SPEED: f32 = 0.1;
pub const SCROLL: f32 = 0.8;

// Простой контроллер 2d камеры
pub fn update(
    time: Res<Time>,
    mut scroll: Local<f32>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut scroll_msg: MessageReader<bevy::input::mouse::MouseWheel>,
    camera: Single<(Mut<Transform>, Mut<Projection>), With<Controller>>,
) {
    let (mut transform, projection) = camera.into_inner();

    for m in scroll_msg.read() {
        *scroll -= m.y * ZOOM_SPEED;
    }
    *scroll = scroll.clamp(-SCROLL, SCROLL);

    let mut velocity = Vec3::ZERO;
    if kbd.pressed(KeyCode::KeyW) {
        velocity.y += 1.0;
    }
    if kbd.pressed(KeyCode::KeyA) {
        velocity.x -= 1.0;
    }
    if kbd.pressed(KeyCode::KeyS) {
        velocity.y -= 1.0;
    }
    if kbd.pressed(KeyCode::KeyD) {
        velocity.x += 1.0;
    }

    let zoom = 1.0 + *scroll;
    match *projection.into_inner() {
        Projection::Orthographic(ref mut orthographic) => {
            orthographic.scale = zoom;
        }
        _ => (),
    };

    if velocity != Vec3::ZERO {
        transform.translation += velocity.normalize() * SPEED * time.delta_secs() * zoom;
    }
}
