use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology::TriangleList, VertexAttributeValues::Float32x3},
};

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum SubVlox {
    X0Y0Z0 = 0,
    X0Y0Z1 = 1,
    X0Y1Z0 = 2,
    X0Y1Z1 = 3,
    X1Y0Z0 = 4,
    X1Y0Z1 = 5,
    X1Y1Z0 = 6,
    X1Y1Z1 = 7,
}

#[derive(Debug)]
pub struct VloxData {
    size: f32,
    root: Vlox,
}
impl VloxData {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            root: Vlox::new(0),
        }
    }
    pub fn get(&self, x: u128, y: u128, z: u128, depth: u8) -> u8 {
        self.root.get(self.xyz_to_path(x, y, z, depth))
    }
    pub fn set(&mut self, x: u128, y: u128, z: u128, depth: u8, value: u8) {
        self.root.set(self.xyz_to_path(x, y, z, depth), value);
    }

    //max depth: 128. Anything more won't be representable as u128.
    fn xyz_to_path(&self, mut x: u128, mut y: u128, mut z: u128, depth: u8) -> Vec<SubVlox> {
        let mut path = vec![];
        let mut blocks = 0; //number of blocks to middle
        for i in 1..(depth + 1) {
            blocks = 2_u128.pow((depth - i) as u32);
            // let x = x % blocks;
            // let y = y % blocks;
            // let z = z % blocks;

            if x < blocks && y < blocks && z < blocks {
                path.push(SubVlox::X0Y0Z0);
            } else if x < blocks && y < blocks && z >= blocks {
                path.push(SubVlox::X0Y0Z1);
                z -= blocks;
            } else if x < blocks && y >= blocks && z < blocks {
                path.push(SubVlox::X0Y1Z0);
                y -= blocks;
            } else if x < blocks && y >= blocks && z >= blocks {
                path.push(SubVlox::X0Y1Z1);
                y -= blocks;
                z -= blocks;
            } else if x >= blocks && y < blocks && z < blocks {
                path.push(SubVlox::X1Y0Z0);
                x -= blocks;
            } else if x >= blocks && y < blocks && z >= blocks {
                path.push(SubVlox::X1Y0Z1);
                x -= blocks;
                z -= blocks;
            } else if x >= blocks && y >= blocks && z < blocks {
                path.push(SubVlox::X1Y1Z0);
                x -= blocks;
                y -= blocks;
            } else if x >= blocks && y >= blocks && z >= blocks {
                path.push(SubVlox::X1Y1Z1);
                x -= blocks;
                y -= blocks;
                z -= blocks;
            }
        }
        path
    }
    pub fn compute_mesh_at_depth(&self, depth: u8) -> Mesh {
        let mut vertices = vec![];
        let mut normals = vec![];
        let mut indices = vec![];

        let mut size = self.size;
        for _ in 0..depth {
            size = size * 0.5;
        }

        let blocks = 2_u128.pow(depth as u32);

        //iterate potential vertices
        for vx in 0..blocks {
            for vy in 0..blocks {
                for vz in 0..blocks {
                    if self.get(vx, vy, vz, depth) == 1 {
                        let x = vx as f32;
                        let y = vy as f32;
                        let z = vz as f32;

                        //right
                        if vx == 0 || self.get(vx - 1, vy, vz, depth) == 0 {
                            vertices.push([size * x, size * y, size * z]);
                            vertices.push([size * x, size * y, size * (z + 1.0)]);
                            vertices.push([size * x, size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * x, size * (y + 1.0), size * z]);
                            normals.push([-1.0, 0.0, 0.0]);
                            normals.push([-1.0, 0.0, 0.0]);
                            normals.push([-1.0, 0.0, 0.0]);
                            normals.push([-1.0, 0.0, 0.0]);
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 4);
                        }
                        //left
                        if vx == blocks - 1 || self.get(vx + 1, vy, vz, depth) == 0 {
                            vertices.push([size * (x + 1.0), size * y, size * z]);
                            vertices.push([size * (x + 1.0), size * y, size * (z + 1.0)]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * z]);
                            normals.push([1.0, 0.0, 0.0]);
                            normals.push([1.0, 0.0, 0.0]);
                            normals.push([1.0, 0.0, 0.0]);
                            normals.push([1.0, 0.0, 0.0]);
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 4);
                        }

                        //bottom
                        if vy == 0 || self.get(vx, vy - 1, vz, depth) == 0 {
                            vertices.push([size * x, size * y, size * z]);
                            vertices.push([size * (x + 1.0), size * y, size * z]);
                            vertices.push([size * (x + 1.0), size * y, size * (z + 1.0)]);
                            vertices.push([size * x, size * y, size * (z + 1.0)]);
                            normals.push([0.0, -1.0, 0.0]);
                            normals.push([0.0, -1.0, 0.0]);
                            normals.push([0.0, -1.0, 0.0]);
                            normals.push([0.0, -1.0, 0.0]);
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 4);
                        }
                        //top
                        if vy == blocks - 1 || self.get(vx, vy + 1, vz, depth) == 0 {
                            vertices.push([size * x, size * (y + 1.0), size * z]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * z]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * x, size * (y + 1.0), size * (z + 1.0)]);
                            normals.push([0.0, 1.0, 0.0]);
                            normals.push([0.0, 1.0, 0.0]);
                            normals.push([0.0, 1.0, 0.0]);
                            normals.push([0.0, 1.0, 0.0]);
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 4);
                        }

                        //back
                        if vz == 0 || self.get(vx, vy, vz - 1, depth) == 0 {
                            vertices.push([size * x, size * y, size * z]);
                            vertices.push([size * x, size * (y + 1.0), size * z]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * z]);
                            vertices.push([size * (x + 1.0), size * y, size * z]);
                            normals.push([0.0, 0.0, -1.0]);
                            normals.push([0.0, 0.0, -1.0]);
                            normals.push([0.0, 0.0, -1.0]);
                            normals.push([0.0, 0.0, -1.0]);
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 4);
                        }
                        //front
                        if vz == blocks - 1 || self.get(vx, vy, vz + 1, depth) == 0 {
                            vertices.push([size * x, size * y, size * (z + 1.0)]);
                            vertices.push([size * x, size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * (x + 1.0), size * y, size * (z + 1.0)]);
                            normals.push([0.0, 0.0, 1.0]);
                            normals.push([0.0, 0.0, 1.0]);
                            normals.push([0.0, 0.0, 1.0]);
                            normals.push([0.0, 0.0, 1.0]);
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 4);
                        }
                    }
                }
            }
        }

        let offset = self.size / 2.0;
        for i in 0..vertices.len() {
            vertices[i][0] -= offset;
            vertices[i][1] -= offset;
            vertices[i][2] -= offset;
        }
        let mut mesh = Mesh::new(TriangleList, RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Float32x3(vertices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Float32x3(normals));
        mesh.insert_indices(Indices::U32(indices));
        mesh
    }
}

#[derive(Clone, Debug)]
struct Vlox {
    value: u8,
    children: Vec<Option<Vlox>>,
}
impl Vlox {
    fn new(value: u8) -> Self {
        Self {
            value,
            children: vec![None; 8],
        }
    }
    fn get(&self, path: Vec<SubVlox>) -> u8 {
        // if we reached the end of the path, return value
        if path.len() == 0 {
            return self.value;
        }
        if self.children.len() == 0 {
            return self.value;
        }
        // if possible, go to the next stage of the path, else return value
        if let Some(child) = &self.children[path[0] as usize] {
            return child.get(path[1..].to_vec());
        } else {
            return self.value;
        }
    }
    fn set(&mut self, path: Vec<SubVlox>, value: u8) {
        // if we reached the end of the path, set value
        if path.len() == 0 {
            self.value = value;
            self.children = vec![];
            return;
        }
        if self.children.len() == 0 {
            self.children = vec![None; 8];
        }
        // go to the next stage of the path, creating a new node if required
        if let Some(child) = &mut self.children[path[0] as usize] {
            child.set(path[1..].to_vec(), value);
        } else {
            let mut vlox = Vlox::new(self.value);
            vlox.set(path[1..].to_vec(), value);
            self.children[path[0] as usize] = Some(vlox);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_vlox_depth_2() {
        let depth: u8 = 2;
        let blocks = 2_u128.pow(depth as u32);

        let mut data = VloxData::new(8.0);

        let mut value = 0;
        for x in 0..blocks {
            for y in 0..blocks {
                for z in 0..blocks {
                    data.set(x, y, z, depth, value);
                    value += 1;
                }
            }
        }

        value = 0;
        for x in 0..blocks {
            for y in 0..blocks {
                for z in 0..blocks {
                    assert_eq!(value, data.get(x, y, z, depth));
                    value += 1;
                }
            }
        }
    }
}
