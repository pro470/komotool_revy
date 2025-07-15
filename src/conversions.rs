use bevy::{math::Vec3A, prelude::*, render::mesh::VertexAttributeValues};
use itertools::Itertools;

// ---

/// Builtin Bevy-to-Rerun conversion methods.
pub trait ToRerun<U> {
    fn to_rerun(&self) -> U;
}

impl ToRerun<rerun::Vec2D> for Vec2 {
    #[inline]
    fn to_rerun(&self) -> rerun::Vec2D {
        rerun::Vec2D::new(self.x, self.y)
    }
}

impl ToRerun<rerun::Vec3D> for Vec3 {
    #[inline]
    fn to_rerun(&self) -> rerun::Vec3D {
        rerun::Vec3D::new(self.x, self.y, self.z)
    }
}
impl ToRerun<rerun::Vec3D> for Vec3A {
    #[inline]
    fn to_rerun(&self) -> rerun::Vec3D {
        rerun::Vec3D::new(self.x, self.y, self.z)
    }
}

impl ToRerun<rerun::Vec4D> for Vec4 {
    #[inline]
    fn to_rerun(&self) -> rerun::Vec4D {
        rerun::Vec4D::new(self.x, self.y, self.z, self.w)
    }
}

impl ToRerun<rerun::Quaternion> for Quat {
    #[inline]
    fn to_rerun(&self) -> rerun::Quaternion {
        rerun::Quaternion::from_xyzw([self.x, self.y, self.z, self.w])
    }
}

impl ToRerun<rerun::Mat3x3> for Mat3 {
    #[inline]
    fn to_rerun(&self) -> rerun::Mat3x3 {
        self.to_cols_array().into()
    }
}

impl ToRerun<rerun::Transform3D> for Transform {
    #[inline]
    fn to_rerun(&self) -> rerun::Transform3D {
        rerun::Transform3D::from_translation_rotation_scale(
            self.translation.to_rerun(),
            self.rotation.to_rerun(),
            rerun::Scale3D::from(self.scale.to_rerun()),
        )
        // Don't show axis - this is quite annoying in Rerun 0.20 otherwise.
        .with_axis_length(0.0)
    }
}
impl ToRerun<rerun::Transform3D> for GlobalTransform {
    #[inline]
    fn to_rerun(&self) -> rerun::Transform3D {
        self.compute_transform().to_rerun()
    }
}

impl ToRerun<rerun::Rgba32> for Color {
    #[inline]
    fn to_rerun(&self) -> rerun::Rgba32 {
        let [r, g, b, a] = self.to_srgba().to_u8_array();
        rerun::Rgba32::from_unmultiplied_rgba(r, g, b, a)
    }
}

impl ToRerun<Option<rerun::archetypes::Mesh3D>> for Mesh {
    #[inline]
    fn to_rerun(&self) -> Option<rerun::archetypes::Mesh3D> {
        if let Some(VertexAttributeValues::Float32x3(positions)) =
            self.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            let mut mesh = rerun::archetypes::Mesh3D::new(positions);

            if let Some(indices) = self.indices() {
                let indices = indices.iter().map(|i| i as u32).collect_vec();
                mesh = mesh
                    .with_triangle_indices(indices.chunks_exact(3).map(|is| [is[0], is[1], is[2]]));
            }

            if let Some(VertexAttributeValues::Float32x3(normals)) =
                self.attribute(Mesh::ATTRIBUTE_NORMAL)
            {
                mesh = mesh.with_vertex_normals(normals);
            }

            if let Some(VertexAttributeValues::Float32x2(texcoords)) =
                self.attribute(Mesh::ATTRIBUTE_UV_0)
            {
                mesh = mesh.with_vertex_texcoords(texcoords);
            }

            if let Some(VertexAttributeValues::Float32x4(colors)) =
                self.attribute(Mesh::ATTRIBUTE_COLOR)
            {
                mesh = mesh.with_vertex_colors(colors.iter().map(|[r, g, b, a]| {
                    // TODO(cmc): is this sRGB? linear? etc?
                    rerun::Color::from_unmultiplied_rgba(
                        (r / 255.0) as u8,
                        (g / 255.0) as u8,
                        (b / 255.0) as u8,
                        (a / 255.0) as u8,
                    )
                }));
            }

            Some(mesh)
        } else {
            None
        }
    }
}
