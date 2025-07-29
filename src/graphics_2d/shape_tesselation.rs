use crate::{
    components_systems::physics_2d::Shape2D,
    graphics_2d::vertex::{ColorVertex, TextureVertex},
};
use cgmath::Vector2;

pub struct TessellatedShape2D {
    pub vertices: Vec<Vector2<f32>>,
    pub indices: Vec<u16>,
}

impl TessellatedShape2D {
    pub fn from(shape: &Shape2D, segments: u32) -> TessellatedShape2D {
        match shape {
            Shape2D::Circle { radius } => Self::circle(*radius, segments),
            Shape2D::Rectangle { half_extents } => Self::rect(half_extents.x, half_extents.y),
        }
    }

    pub fn outline_from(shape: &Shape2D, thickness: f32, segments: u32) -> TessellatedShape2D {
        match shape {
            Shape2D::Circle { radius } => Self::circle_outline(*radius, thickness, segments),
            Shape2D::Rectangle { half_extents } => {
                Self::rect_outline(half_extents.x, half_extents.y, thickness)
            }
        }
    }

    pub fn circle(radius: f32, segments: u32) -> Self {
        let mut vertices = Vec::with_capacity((segments + 1) as usize);
        let mut indices = Vec::with_capacity((segments * 3) as usize);

        // Center point
        vertices.push(Vector2::new(0.0, 0.0));

        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = radius * theta.cos();
            let y = radius * theta.sin();

            vertices.push(Vector2::new(x, y));

            if i < segments {
                indices.push(0);
                indices.push(i as u16 + 1);
                indices.push(i as u16 + 2);
            }
        }

        Self { vertices, indices }
    }

    pub fn rect(hw: f32, hh: f32) -> Self {
        let vertices = vec![
            Vector2::new(-hw, hh),
            Vector2::new(hw, hh),
            Vector2::new(hw, -hh),
            Vector2::new(-hw, -hh),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Self { vertices, indices }
    }

    /// Generates a ring (outline) of a circle as a triangle strip.
    /// `thickness` is the outline thickness.
    pub fn circle_outline(radius: f32, thickness: f32, segments: u32) -> Self {
        let mut vertices = Vec::with_capacity((segments * 2 + 2) as usize);
        let mut indices = Vec::with_capacity((segments * 6) as usize);

        let half_thickness = thickness / 2.0;

        for i in 0..segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();

            // Outer vertex
            vertices.push(Vector2::new(
                (radius + half_thickness) * cos_theta,
                (radius + half_thickness) * sin_theta,
            ));
            // Inner vertex
            vertices.push(Vector2::new(
                (radius - half_thickness) * cos_theta,
                (radius - half_thickness) * sin_theta,
            ));
        }
        // Close the loop by repeating first two vertices
        vertices.push(vertices[0]);
        vertices.push(vertices[1]);

        // Build indices for triangle strip as triangles
        for i in 0..segments {
            let i0 = (i * 2) as u16;
            indices.push(i0);
            indices.push(i0 + 1);
            indices.push(i0 + 2);

            indices.push(i0 + 2);
            indices.push(i0 + 1);
            indices.push(i0 + 3);
        }

        Self { vertices, indices }
    }

    /// Generates a rectangle outline as a ring (two rectangles forming a thick border).
    /// `thickness` is the outline thickness.
    pub fn rect_outline(hw: f32, hh: f32, thickness: f32) -> Self {
        let mut vertices = Vec::with_capacity(8);
        let mut indices = Vec::with_capacity(18); // 6 triangles * 3 indices

        let outer_hw = hw + thickness / 2.0;
        let outer_hh = hh + thickness / 2.0;
        let inner_hw = hw - thickness / 2.0;
        let inner_hh = hh - thickness / 2.0;

        // Outer rectangle (clockwise)
        vertices.push(Vector2::new(-outer_hw, -outer_hh)); // 0
        vertices.push(Vector2::new(outer_hw, -outer_hh)); // 1
        vertices.push(Vector2::new(outer_hw, outer_hh)); // 2
        vertices.push(Vector2::new(-outer_hw, outer_hh)); // 3

        // Inner rectangle (clockwise)
        vertices.push(Vector2::new(-inner_hw, -inner_hh)); // 4
        vertices.push(Vector2::new(inner_hw, -inner_hh)); // 5
        vertices.push(Vector2::new(inner_hw, inner_hh)); // 6
        vertices.push(Vector2::new(-inner_hw, inner_hh)); // 7

        // Build indices for two rectangle rings (two triangles per side, 4 sides)
        // We'll build two triangles per side connecting outer and inner rects

        // Side 1 (bottom)
        indices.extend_from_slice(&[0, 4, 5, 0, 5, 1]);
        // Side 2 (right)
        indices.extend_from_slice(&[1, 5, 6, 1, 6, 2]);
        // Side 3 (top)
        indices.extend_from_slice(&[2, 6, 7, 2, 7, 3]);
        // Side 4 (left)
        indices.extend_from_slice(&[3, 7, 4, 3, 4, 0]);

        Self { vertices, indices }
    }

    pub fn apply_color(&self, color: [f32; 4]) -> Vec<ColorVertex> {
        self.vertices
            .iter()
            .map(|pos| ColorVertex {
                position: [pos.x, pos.y],
                color,
            })
            .collect()
    }

    pub fn into_tex(&self, tex_coords: [[f32; 2]; 4]) -> Vec<TextureVertex> {
        self.vertices
            .iter()
            .zip(tex_coords.iter())
            .map(|(v, &uv)| TextureVertex {
                position: [v.x, v.y],
                tex_coords: uv,
            })
            .collect()
    }

    pub fn recenter(&mut self, center: Vector2<f32>) -> &mut Self {
        if self.vertices.is_empty() {
            return self;
        }

        // Compute current centroid (average of positions)
        let sum = self
            .vertices
            .iter()
            .fold(Vector2::new(0.0, 0.0), |acc, pos| acc + *pos);
        let count = self.vertices.len() as f32;
        let current_centroid = sum / count;

        // Compute translation vector: how much to move all positions
        let translation = center - current_centroid;

        // Apply translation to all positions
        for pos in &mut self.vertices {
            *pos += translation;
        }
        self
    }
}
