use nalgebra::Vector3;
use simdnoise::{CellDistanceFunction, CellReturnType};
use std::f32::{MAX, MIN};

#[derive(Serialize, Deserialize)]
#[serde(remote = "CellDistanceFunction")]
enum CellDistanceFunctionDef {
    Euclidean,
    Manhattan,
    Natural,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "CellReturnType")]
enum CellReturnTypeDef {
    CellValue,
    Distance,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TerrainLayer {
    Add(Vec<TerrainLayer>),
    Multiply(Vec<TerrainLayer>),
    Constant(f32),
    Clamp {min:Option<f32>, max:Option<f32>, value:Box<TerrainLayer>},
    NoiseCellular {
        #[serde(with = "CellDistanceFunctionDef")]
        distance_fn: CellDistanceFunction,
        #[serde(with = "CellReturnTypeDef")]
        return_type: CellReturnType,
        jitter: f32
    },
    NoiseFBM { freq: f32, lacunarity: f32, gain: f32, octaves: u8 },
    NoiseRidge { freq: f32, lacunarity: f32, gain: f32, octaves: u8 },
    NoiseSimplex,
    NoiseTurbulence { freq: f32, lacunarity: f32, gain: f32, octaves: u8 }
}

impl TerrainLayer {
    pub fn compute_height(&self, dir: &Vector3<f32>) -> f32 {
        match self {
            TerrainLayer::Add (children) => {
                let mut height : f32 = 0.0;
                for child in children {
                    let child_height = child.compute_height(dir);
                    height += child_height;
                }
                height
            },
            TerrainLayer::Multiply (children) => {
                let mut height : f32 = 1.0;
                for child in children {
                    let child_height = child.compute_height(dir);
                    height *= child_height;
                }
                height
            },
            TerrainLayer::Clamp {min, max, value } => {
                value.compute_height(dir)
                    .min(max.unwrap_or(MAX ))
                    .max( min.unwrap_or( MIN ))
            }
            TerrainLayer::Constant(height) => *height,
            TerrainLayer::NoiseCellular { distance_fn, return_type, jitter } => {
                simdnoise::scalar::cellular_3d(dir.x, dir.y, dir.z, *distance_fn, *return_type, *jitter)
            },
            TerrainLayer::NoiseFBM { freq, lacunarity, gain, octaves } => {
                simdnoise::scalar::fbm_3d(dir.x, dir.y, dir.z, *freq, *lacunarity, *gain, *octaves)
            },
            TerrainLayer::NoiseRidge { freq, lacunarity, gain, octaves } => {
                simdnoise::scalar::ridge_3d(dir.x, dir.y, dir.z, *freq, *lacunarity, *gain, *octaves)
            },
            TerrainLayer::NoiseSimplex => {
                simdnoise::scalar::simplex_3d(dir.x, dir.y, dir.z)
            },
            TerrainLayer::NoiseTurbulence { freq, lacunarity, gain, octaves } => {
                simdnoise::scalar::turbulence_3d(dir.x, dir.y, dir.z, *freq, *lacunarity, *gain, *octaves)
            }
        }
    }
}