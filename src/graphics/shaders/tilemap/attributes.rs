use cgmath::{Vector2, Vector3, Zero};
use std::path::PathBuf;

pub struct TilemapAttributes {
    texture_path: PathBuf,
    chunks: Vec<TilemapChunk>,
    chunk_size: u16,
    position: Vector3<f32>,
    default_tile: Tile,
}

struct TilemapChunk {
    pub tiles: Vec<Tile>,
    pub position: ChunkPosition,
}

pub struct CreateTilemapChunkInput<'a> {
    pub position: ChunkPosition,
    pub default_tile: Option<Tile>,
    pub initial_tiles: &'a [(Tile, ChunkedTilePosition)],
}

type Tile = u32;
type ChunkPosition = Vector3<i64>;
type ChunkedTilePosition = Vector2<u16>;
type GlobalTilePosition = Vector3<i64>;

impl TilemapAttributes {
    pub fn new<P>(texture_path: P, chunk_size: u16, default_tile: Tile) -> Self
    where
        P: Into<PathBuf>,
    {
        if chunk_size == 0 {
            panic!("Chunk size must be greater than 0");
        }

        Self {
            texture_path: texture_path.into(),
            chunks: vec![],
            chunk_size,
            position: Vector3::zero(),
            default_tile,
        }
    }

    pub fn texture_path(&self) -> &PathBuf {
        &self.texture_path
    }

    pub fn set_tile_or_create_chunk(&mut self, position: &GlobalTilePosition, tile: Tile) {
        let result = self.set_tile(position, tile);
        match result {
            Err(_) => {
                let _ = self.create_chunk(&CreateTilemapChunkInput {
                    position: self.chunk_pos_from_tile_global_pos(position),
                    default_tile: None,
                    initial_tiles: &[],
                });
            }
            _ => {}
        }
    }

    pub fn set_tile_or<F>(&mut self, position: &GlobalTilePosition, tile: Tile, callback: F)
    where
        F: Fn(),
    {
        let _ = self.set_tile(position, tile).inspect_err(|_| {
            callback();
        });
    }

    pub fn set_tile(&mut self, position: &GlobalTilePosition, tile: Tile) -> Result<(), ()> {
        let tile_index = self.tile_global_pos_to_index(position);
        let chunk_index = self.chunk_index_from_tile_pos(position).ok_or(())?;
        let chunk = &mut self.chunks[chunk_index];
        chunk.tiles[tile_index] = tile;

        Ok(())
    }

    fn chunk_index_from_tile_pos(&self, position: &GlobalTilePosition) -> Option<usize> {
        for (index, chunk) in self.chunks.iter().enumerate() {
            if self.is_position_in_chunk(&position, &chunk) {
                return Some(index);
            }
        }
        None
    }

    pub fn create_chunk(&mut self, input: &CreateTilemapChunkInput) -> Result<(), ()> {
        for chunk in &mut self.chunks {
            if chunk.position == input.position {
                return Err(());
            }
        }

        let default_tile = input.default_tile.unwrap_or(self.default_tile);
        let mut chunk = TilemapChunk {
            tiles: vec![default_tile; self.chunk_size.pow(2) as usize],
            position: input.position,
        };

        for (tile, position) in input.initial_tiles {
            let index = self.tile_chunk_pos_to_index(position);
            chunk.tiles[index] = *tile;
        }

        self.chunks.push(chunk);
        Ok(())
    }

    pub fn delete_chunks(&mut self, positions: &[ChunkPosition]) {
        let mut indices_to_remove = Vec::with_capacity(positions.len());
        for (index, chunk) in self.chunks.iter().enumerate() {
            if positions.contains(&chunk.position) {
                indices_to_remove.push(index);
            }
        }

        for index in indices_to_remove {
            self.chunks.swap_remove(index);
        }
    }

    fn is_position_in_chunk(&self, position: &GlobalTilePosition, chunk: &TilemapChunk) -> bool {
        let max_x = chunk.position.x + self.chunk_size as i64;
        let max_y = chunk.position.y + self.chunk_size as i64;
        return position.z == chunk.position.z
            && position.x > chunk.position.x
            && position.x < max_x
            && position.y > chunk.position.y
            && position.y < max_y;
    }

    fn tile_global_pos_to_index(&self, position: &GlobalTilePosition) -> usize {
        todo!()
    }

    fn tile_chunk_pos_to_index(&self, position: &ChunkedTilePosition) -> usize {
        todo!()
    }

    fn chunk_pos_from_tile_global_pos(&self, position: &GlobalTilePosition) -> ChunkPosition {
        todo!()
    }

    pub fn iter_tiles(&self) -> TilemapTileIterator {
        TilemapTileIterator::new(self)
    }
}
pub struct TilemapTileIterator<'a> {
    attributes: &'a TilemapAttributes,
    current_chunk_index: usize,
    current_tile_index: usize,
}

pub struct TilemapTileIteratorItem {
    pub chunk_position: ChunkPosition,
    pub tile_chunk_position: ChunkedTilePosition,
    pub tile_global_position: GlobalTilePosition,
    pub tile: Tile,
}

impl<'a> TilemapTileIterator<'a> {
    pub fn new(attributes: &'a TilemapAttributes) -> Self {
        Self {
            attributes,
            current_chunk_index: 0,
            current_tile_index: 0,
        }
    }
}

impl<'a> Iterator for TilemapTileIterator<'a> {
    type Item = TilemapTileIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        let max_chunk_index = self.attributes.chunks.len() - 1;
        let max_tile_index = (self.attributes.chunk_size - 1) as usize;

        if self.current_chunk_index > max_chunk_index {
            return None;
        }
        if self.current_tile_index > max_tile_index {
            self.current_chunk_index += 1;
        }

        let chunk = &self.attributes.chunks[self.current_chunk_index];
        let tile = chunk.tiles[self.current_tile_index];

        self.current_tile_index += 1;

        let item = Self::Item {
            chunk_position: chunk.position,
            tile_chunk_position: todo!(),
            tile_global_position: todo!(),
            tile,
        };
        Some(item)
    }
}
