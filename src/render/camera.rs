use bevy::prelude::*;

// Компонент-маркер для нашей камеры
#[derive(Component)]
pub struct FlyCamera {
    pub speed: f32,
}

// Система управления камерой (WASD + Space/Shift)
pub fn fly_camera_system(
    time: Res<Time<Real>>, // Добавили <Real>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&FlyCamera, &mut Transform)>,
) {
    let delta = time.delta_secs(); // Теперь заработает
    for (cam, mut transform) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;

        // Получаем векторы направления
        let forward = transform.forward().normalize_or_zero();
        let right = transform.right().normalize_or_zero();

        if keyboard_input.pressed(KeyCode::KeyW) { velocity += forward; }
        if keyboard_input.pressed(KeyCode::KeyS) { velocity -= forward; }
        if keyboard_input.pressed(KeyCode::KeyA) { velocity -= right; }
        if keyboard_input.pressed(KeyCode::KeyD) { velocity += right; }
        if keyboard_input.pressed(KeyCode::Space) { velocity += Vec3::Y; }
        if keyboard_input.pressed(KeyCode::ShiftLeft) { velocity -= Vec3::Y; }

        // Обновляем позицию
        transform.translation += velocity * cam.speed * time.delta_secs();

        // Простое управление поворотом на стрелочки (или можно прикрутить мышь)
        if keyboard_input.pressed(KeyCode::ArrowLeft) { transform.rotate_y(1.0 * time.delta_secs()); }
        if keyboard_input.pressed(KeyCode::ArrowRight) { transform.rotate_y(-1.0 * time.delta_secs()); }
        if keyboard_input.pressed(KeyCode::ArrowUp) { transform.rotate_local_x(1.0 * time.delta_secs()); }
        if keyboard_input.pressed(KeyCode::ArrowDown) { transform.rotate_local_x(-1.0 * time.delta_secs()); }
    }
}

// Добавь это в camera.rs
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCamera { speed: 50.0 }, // Твой компонент из этого же файла
    ));

}