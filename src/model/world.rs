use Entity;

pub struct World {
    regions: Map<Vector2<u64>, Region>,
    entities: Map<u64, Entity>,
}

pub struct Region {
    chunks: Map<Vector2<u64>, Chunk>,
}

pub struct Chunk {
    blocks: [[[Block;32];32];32],
}

pub struct Block {
    material: Material,
}

pub struct Material {
    resistance: f32,
    opacity: f32,
}
