use std::collections::HashMap;

pub type MaterialId = u16;

#[derive(Default)]
pub struct MaterialMap {
    map: HashMap<MaterialId, Material>,
}
impl MaterialMap {
    pub fn color(&self, id: MaterialId, vx: u128, vy: u128, vz: u128, depth: u8) -> VloxColor {
        match &self.map[&id] {
            Material::Void => VloxColor::Void,
            Material::Solid(builder) => {
                let color_index = builder.data.get(vx, vy, vz, depth);
                VloxColor::Solid(builder.colors[color_index as usize])
            }
            Material::Custom(builder) => {
                //TODO: call wasm to get real materialid, then lookup color
                VloxColor::Void
            }
        }
    }
    pub fn get(&self, id: MaterialId) -> Option<&Material> {
        self.map.get(&id)
    }
    pub fn set(&mut self, id: MaterialId, material: Material) {
        self.map.insert(id, material);
    }
}

#[derive(PartialEq)]
pub enum VloxColor {
    Void,
    Solid(Color),
}

pub enum Material {
    Void,
    Solid(SolidMaterial),
    Custom(CustomMaterial),
}
pub struct SolidMaterial {
    pub name: String,
    pub data: VloxData,
    pub colors: Vec<Color>,
}
pub struct CustomMaterial {
    pub name: String,
    pub wasm: Vec<u8>,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub fn as_f32x4(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

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
impl Default for VloxData {
    fn default() -> Self {
        Self::new(0)
    }
}
impl VloxData {
    pub fn new(depth_to_unit: u8) -> Self {
        let size = 2_u128.pow(depth_to_unit as u32) as f32;
        Self {
            size,
            root: Vlox::new(0),
        }
    }
    pub fn size(&self) -> f32 {
        self.size
    }
    pub fn num_vlox(&self, depth: u8) -> u128 {
        2_u128.pow(depth as u32)
    }
    pub fn vlox_size(&self, num_vlox: u128) -> f32 {
        self.size / num_vlox as f32
    }
    pub fn xyz_f32_to_vlox_xyz(&self, x: f32, y: f32, z: f32, depth: u8) -> (u128, u128, u128) {
        let num_vlox = self.num_vlox(depth) as f32;
        let offset = self.size * 0.5;
        (
            ((x + offset) / self.size * num_vlox) as u128,
            ((y + offset) / self.size * num_vlox) as u128,
            ((z + offset) / self.size * num_vlox) as u128,
        )
    }
    pub fn vlox_xyz_to_xyz_f32(&self, vx: u128, vy: u128, vz: u128, depth: u8) -> (f32, f32, f32) {
        let num_vlox = self.num_vlox(depth);
        let half_vlox = self.vlox_size(num_vlox) * 0.5;
        let offset = self.size * 0.5;
        (
            (vx as f32 / num_vlox as f32 * self.size - offset + half_vlox),
            (vy as f32 / num_vlox as f32 * self.size - offset + half_vlox),
            (vz as f32 / num_vlox as f32 * self.size - offset + half_vlox),
        )
    }
    pub fn get(&self, x: u128, y: u128, z: u128, depth: u8) -> MaterialId {
        self.root.get(self.xyz_to_path(x, y, z, depth))
    }
    pub fn set(&mut self, x: u128, y: u128, z: u128, depth: u8, value: MaterialId) {
        self.root.set(self.xyz_to_path(x, y, z, depth), value);
    }

    //max depth: 128. Anything more won't be representable as u128.
    fn xyz_to_path(&self, mut x: u128, mut y: u128, mut z: u128, depth: u8) -> Vec<SubVlox> {
        let mut path = vec![];
        let mut blocks; //number of blocks to middle
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
    pub fn compute_mesh_at_depth(
        &self,
        depth: u8,
        materials: &MaterialMap,
    ) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 4]>, Vec<u32>) {
        let mut vertices = vec![];
        let mut normals = vec![];
        let mut colors = vec![];
        let mut indices = vec![];

        let mut size = self.size;
        for _ in 0..depth {
            size = size * 0.5;
        }

        let blocks = 2_u128.pow(depth as u32);

        //iterate potential vertices
        let mut x;
        let mut y;
        let mut z;
        let mut id;
        let mut adjacent_id;
        let mut adjacent_color;
        for vx in 0..blocks {
            for vy in 0..blocks {
                for vz in 0..blocks {
                    id = self.get(vx, vy, vz, depth);
                    if let VloxColor::Solid(color) = materials.color(id, 0, 0, 0, 0) {
                        x = vx as f32;
                        y = vy as f32;
                        z = vz as f32;

                        //right
                        adjacent_id = self.get(vx - 1, vy, vz, depth);
                        adjacent_color = materials.color(adjacent_id, 0, 0, 0, 0);
                        if vx == 0 || adjacent_color == VloxColor::Void {
                            vertices.push([size * x, size * y, size * z]);
                            vertices.push([size * x, size * y, size * (z + 1.0)]);
                            vertices.push([size * x, size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * x, size * (y + 1.0), size * z]);
                            normals.push([-1.0, 0.0, 0.0]);
                            normals.push([-1.0, 0.0, 0.0]);
                            normals.push([-1.0, 0.0, 0.0]);
                            normals.push([-1.0, 0.0, 0.0]);
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 4);
                        }
                        //left
                        adjacent_id = self.get(vx + 1, vy, vz, depth);
                        adjacent_color = materials.color(adjacent_id, 0, 0, 0, 0);
                        if vx == blocks - 1 || adjacent_color == VloxColor::Void {
                            vertices.push([size * (x + 1.0), size * y, size * z]);
                            vertices.push([size * (x + 1.0), size * y, size * (z + 1.0)]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * z]);
                            normals.push([1.0, 0.0, 0.0]);
                            normals.push([1.0, 0.0, 0.0]);
                            normals.push([1.0, 0.0, 0.0]);
                            normals.push([1.0, 0.0, 0.0]);
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 4);
                        }

                        //bottom
                        adjacent_id = self.get(vx, vy - 1, vz, depth);
                        adjacent_color = materials.color(adjacent_id, 0, 0, 0, 0);
                        if vy == 0 || adjacent_color == VloxColor::Void {
                            vertices.push([size * x, size * y, size * z]);
                            vertices.push([size * (x + 1.0), size * y, size * z]);
                            vertices.push([size * (x + 1.0), size * y, size * (z + 1.0)]);
                            vertices.push([size * x, size * y, size * (z + 1.0)]);
                            normals.push([0.0, -1.0, 0.0]);
                            normals.push([0.0, -1.0, 0.0]);
                            normals.push([0.0, -1.0, 0.0]);
                            normals.push([0.0, -1.0, 0.0]);
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 4);
                        }
                        //top
                        adjacent_id = self.get(vx, vy + 1, vz, depth);
                        adjacent_color = materials.color(adjacent_id, 0, 0, 0, 0);
                        if vy == blocks - 1 || adjacent_color == VloxColor::Void {
                            vertices.push([size * x, size * (y + 1.0), size * z]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * z]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * x, size * (y + 1.0), size * (z + 1.0)]);
                            normals.push([0.0, 1.0, 0.0]);
                            normals.push([0.0, 1.0, 0.0]);
                            normals.push([0.0, 1.0, 0.0]);
                            normals.push([0.0, 1.0, 0.0]);
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 4);
                        }

                        //back
                        adjacent_id = self.get(vx, vy, vz - 1, depth);
                        adjacent_color = materials.color(adjacent_id, 0, 0, 0, 0);
                        if vz == 0 || adjacent_color == VloxColor::Void {
                            vertices.push([size * x, size * y, size * z]);
                            vertices.push([size * x, size * (y + 1.0), size * z]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * z]);
                            vertices.push([size * (x + 1.0), size * y, size * z]);
                            normals.push([0.0, 0.0, -1.0]);
                            normals.push([0.0, 0.0, -1.0]);
                            normals.push([0.0, 0.0, -1.0]);
                            normals.push([0.0, 0.0, -1.0]);
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            indices.push(vertices.len() as u32 - 4);
                            indices.push(vertices.len() as u32 - 3);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 2);
                            indices.push(vertices.len() as u32 - 1);
                            indices.push(vertices.len() as u32 - 4);
                        }
                        //front
                        adjacent_id = self.get(vx, vy, vz + 1, depth);
                        adjacent_color = materials.color(adjacent_id, 0, 0, 0, 0);
                        if vz == blocks - 1 || adjacent_color == VloxColor::Void {
                            vertices.push([size * x, size * y, size * (z + 1.0)]);
                            vertices.push([size * x, size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * (x + 1.0), size * (y + 1.0), size * (z + 1.0)]);
                            vertices.push([size * (x + 1.0), size * y, size * (z + 1.0)]);
                            normals.push([0.0, 0.0, 1.0]);
                            normals.push([0.0, 0.0, 1.0]);
                            normals.push([0.0, 0.0, 1.0]);
                            normals.push([0.0, 0.0, 1.0]);
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
                            colors.push(color.as_f32x4());
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
        (vertices, normals, colors, indices)
    }
}

#[derive(Clone, Debug)]
struct Vlox {
    value: MaterialId,
    children: Vec<Option<Vlox>>,
}
impl Vlox {
    fn new(value: MaterialId) -> Self {
        Self {
            value,
            children: vec![None; 8],
        }
    }
    fn get(&self, path: Vec<SubVlox>) -> MaterialId {
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
    fn set(&mut self, path: Vec<SubVlox>, value: MaterialId) {
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

        let mut data = VloxData::new(3);

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
                    assert_eq!(value, data.get(x * 2, y * 2, z * 2, depth + 1));
                    assert_eq!(value, data.get(x * 2, y * 2, z * 2 + 1, depth + 1));
                    assert_eq!(value, data.get(x * 2, y * 2 + 1, z * 2, depth + 1));
                    assert_eq!(value, data.get(x * 2, y * 2 + 1, z * 2 + 1, depth + 1));
                    assert_eq!(value, data.get(x * 2 + 1, y * 2, z * 2, depth + 1));
                    assert_eq!(value, data.get(x * 2 + 1, y * 2, z * 2 + 1, depth + 1));
                    assert_eq!(value, data.get(x * 2 + 1, y * 2 + 1, z * 2, depth + 1));
                    assert_eq!(value, data.get(x * 2 + 1, y * 2 + 1, z * 2 + 1, depth + 1));
                    value += 1;
                }
            }
        }
    }
}
