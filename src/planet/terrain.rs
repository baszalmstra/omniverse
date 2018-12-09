use nalgebra::Vector3;
use simdnoise::{CellDistanceFunction, CellReturnType};

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

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum TerrainCombinator {
    Add,
    Multiply
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum NoiseFunction {
    Cellular {
        #[serde(with = "CellDistanceFunctionDef")]
        distance_fn: CellDistanceFunction,
        #[serde(with = "CellReturnTypeDef")]
        return_type: CellReturnType,
        jitter: f32
    },
    FBM { freq: f32, lacunarity: f32, gain: f32, octaves: u8 },
    Ridge { freq: f32, lacunarity: f32, gain: f32, octaves: u8 },
    Simplex,
    Turbulence { freq: f32, lacunarity: f32, gain: f32, octaves: u8 }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TerrainFunction {
    Children { op: TerrainCombinator, children: Vec<TerrainLayer> },
    Constant(f32),
    Noise(NoiseFunction)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TerrainLayer {
    func: TerrainFunction,
    low: Option<f32>,
    high: Option<f32>
}

impl TerrainLayer {
    pub fn new(func: TerrainFunction) -> TerrainLayer {
        TerrainLayer {
            func,
            low: None,
            high: None
        }
    }

    pub fn set_low(&mut self, low: f32) -> &mut Self {
        self.low = Some(low);
        self
    }

    pub fn set_high(&mut self, high: f32) -> &mut Self {
        self.high = Some(high);
        self
    }

    pub fn compute_height(&self, dir: &Vector3<f32>) -> f32 {
        match &self.func {
            TerrainFunction::Children { op, children } => {
                let mut height : f32 = if let TerrainCombinator::Multiply = op { 1.0 } else { 0.0 };
                for child in children {
                    let child_height = child.compute_height(dir);
                    match op {
                        TerrainCombinator::Add => height += child_height,
                        TerrainCombinator::Multiply => height *= child_height
                    }
                }
                height
            },
            TerrainFunction::Constant(height) => *height,
            TerrainFunction::Noise(function) => match function {
                NoiseFunction::Cellular { distance_fn, return_type, jitter } => {
                    simdnoise::scalar::cellular_3d(dir.x, dir.y, dir.z, *distance_fn, *return_type, *jitter)
                },
                NoiseFunction::FBM { freq, lacunarity, gain, octaves } => {
                    simdnoise::scalar::fbm_3d(dir.x, dir.y, dir.z, *freq, *lacunarity, *gain, *octaves)
                },
                NoiseFunction::Ridge { freq, lacunarity, gain, octaves } => {
                    simdnoise::scalar::ridge_3d(dir.x, dir.y, dir.z, *freq, *lacunarity, *gain, *octaves)
                },
                NoiseFunction::Simplex => {
                    simdnoise::scalar::simplex_3d(dir.x, dir.y, dir.z)
                },
                NoiseFunction::Turbulence { freq, lacunarity, gain, octaves } => {
                    simdnoise::scalar::turbulence_3d(dir.x, dir.y, dir.z, *freq, *lacunarity, *gain, *octaves)
                }
            }
        }
    }
}