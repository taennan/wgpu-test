use glam::{Mat4, Quat, Vec3};

// ---- Translation ----

/// Moves `position` by `offset`, measured along world/global axes.
pub fn translate_global(position: Vec3, offset: Vec3) -> Vec3 {
    position + offset
}

/// Moves `position` by `offset`, measured along the object's own local axes
/// (as defined by `rotation`). e.g. "move forward" should use this, since
/// forward means different things depending on which way the object faces.
pub fn translate_local(position: Vec3, offset: Vec3, rotation: Quat) -> Vec3 {
    position + rotation * offset
}

// ---- Rotation ----

/// Rotates `rotation` by Euler angle `offset` (radians, XYZ order), applied
/// around the world's fixed axes. Order matters: the delta is applied
/// *before* the existing rotation.
pub fn rotate_global(rotation: Quat, euler_offset: Vec3) -> Quat {
    let delta = Quat::from_euler(
        glam::EulerRot::XYZ,
        euler_offset.x,
        euler_offset.y,
        euler_offset.z,
    );
    (delta * rotation).normalize()
}

/// Rotates `rotation` by Euler angle `offset` (radians, XYZ order), applied
/// around the object's own current axes. Order matters: the delta is
/// applied *after* the existing rotation.
pub fn rotate_local(rotation: Quat, euler_offset: Vec3) -> Quat {
    let delta = Quat::from_euler(
        glam::EulerRot::XYZ,
        euler_offset.x,
        euler_offset.y,
        euler_offset.z,
    );
    (rotation * delta).normalize()
}

// ---- Direction vectors ----
// Convention: forward is -Z, right is +X, up is +Y, matching typical
// right-handed OpenGL/wgpu conventions (pairs with look_at_rh / perspective_rh).

pub fn forward(rotation: Quat) -> Vec3 {
    rotation * Vec3::NEG_Z
}

pub fn right(rotation: Quat) -> Vec3 {
    rotation * Vec3::X
}

pub fn up(rotation: Quat) -> Vec3 {
    rotation * Vec3::Y
}

// ---- Look-at ----

/// Computes the rotation that orients an object at `position` to face
/// `target`, using `up_reference` as the "up" hint. Returns `None` if
/// `target` is (numerically) at `position`, since direction is undefined.
///
/// Note: if the forward direction ends up parallel to `up_reference` (e.g.
/// looking straight up/down), roll around the forward axis is arbitrary —
/// there's no unique answer, not a bug. Pass in a different `up_reference`
/// if you need deterministic behavior in that case.
pub fn look_at(position: Vec3, target: Vec3, up_reference: Vec3) -> Option<Quat> {
    let diff = target - position;
    if diff.length_squared() < 1e-12 {
        return None;
    }
    let fwd = diff.normalize();

    let up_ref = if fwd.abs_diff_eq(up_reference.normalize(), 1e-4)
        || fwd.abs_diff_eq(-up_reference.normalize(), 1e-4)
    {
        // Fall back to a reference axis that isn't parallel to `fwd`.
        if fwd.abs_diff_eq(Vec3::X, 1e-4) || fwd.abs_diff_eq(-Vec3::X, 1e-4) {
            Vec3::Y
        } else {
            Vec3::X
        }
    } else {
        up_reference
    };

    let right = fwd.cross(up_ref).normalize();
    let up = right.cross(fwd);

    let rotation_matrix = Mat4::from_cols(
        right.extend(0.0),
        up.extend(0.0),
        (-fwd).extend(0.0),
        glam::Vec4::W,
    );

    Some(Quat::from_mat4(&rotation_matrix).normalize())
}
