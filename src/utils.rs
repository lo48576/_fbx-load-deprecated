//! Contains utilities useful (but not required) to use with FBX data.

/// Triangulates single polygon and returns the number of new triangles.
///
/// Triangulate the polygon (`vertices[poly_indices[0]]`, .., `vertices[poly_indices[n]]`)
/// (where `n` is `poly_indices.len()`) into (`vertices[poly_indices[triangulated[0]]]`, ..,
/// `vertices[poly_indices[triangulated[3*m+2]]]`) (whene `m` is the number of new triangles)
/// and push `[triangulated[0], .., triangulated[3*m+2]]` to the `target`.
pub fn triangulate_polygon(vertices: &[[f32; 3]], poly_indices: &[u32], target: &mut Vec<u32>) -> u32 {
    let vec_cross = |v1: &[f32; 3], v2: &[f32; 3]| {
        [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[1],
            v1[0] * v2[1] - v1[1] * v2[1],
        ]
    };
    let vec_sub = |v1: &[f32; 3], v2: &[f32; 3]| {
        [
            v1[0] - v2[0],
            v1[1] - v2[1],
            v1[2] - v2[2],
        ]
    };
    let vec_dot = |v1: &[f32; 3], v2: &[f32; 3]| {
        v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
    };

    match poly_indices.len() {
        n@0...2 => {
            // Not an FBX polygon!
            // FBX polygon must have 3 or more vertices.
            warn!("Attempt to triangulate non-polygon: {}-gon", n);
            0
        },
        3 => {
            // Given a triangle, no need of triangulation.
            target.extend_from_slice(&[0, 1, 2]);
            1
        },
        4 => {
            // Optimize for quadrangles.
            // p0, p1, p2, p3: vertices of the quadrangle.
            let p0 = &vertices[poly_indices[0] as usize];
            let p1 = &vertices[poly_indices[1] as usize];
            let p2 = &vertices[poly_indices[2] as usize];
            let p3 = &vertices[poly_indices[3] as usize];
            // n1: Normal vector of quadrangle calculated with two edges of the angle1
            // n3: Normal vector of quadrangle calculated with two edges of the angle3
            let n1 = vec_cross(&vec_sub(p0, p1), &vec_sub(p1, p2));
            let n3 = vec_cross(&vec_sub(p2, p3), &vec_sub(p3, p0));
            // If both angle1 and angle3 are concave, vectors n1 and n3 are oriented in the same
            // direction and dot(n1, n3) will be positive.
            // If either angle1 or angle3 is concave, vector n1 and n3 are oriented in the opposite
            // directions and dot(n1, n3) will be negative.
            // It does not matter when the vertices of quadrangle is not on the same plane,
            // because whichever diagonal you choose, the cut will be inaccurate.
            if vec_dot(&n1, &n3) >= 0.0 {
                // Both angle1 and angle3 are concave.
                // Cut from p0 to p2.
                target.extend_from_slice(&[0, 1, 2, 2, 3, 0]);
            } else {
                // Either angle1 or angle3 is convex.
                // Cut from p1 to p3.
                target.extend_from_slice(&[0, 1, 3, 3, 1, 2]);
            }
            2
        },
        n => {
            // TODO: Support polygons with 0 or 1 convex angles. It would not be difficult.
            warn!("Unsupported polygon: {}-gon", n);
            0
        },
    }
}
