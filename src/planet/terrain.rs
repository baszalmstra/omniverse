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
    NoiseFBM { frequency: f32, persistence: f32, octaves: usize },
    NoiseRidge { frequency: f32, persistence: f32, octaves: usize },
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
            TerrainLayer::NoiseFBM { frequency: freq, persistence, octaves } => {
                let mut result = 0.0;
                let mut max_amplitude = 0.0;
                let mut amplitide = 1.0;
                let mut frequency = *freq;
                for _ in 0..*octaves {
                    result += ((1.0 - simdnoise::scalar::simplex_3d(dir.x*frequency, dir.y*frequency, dir.z*frequency).abs()) * 2.0 - 1.0) * amplitide;
                    frequency *= 2.0;
                    max_amplitude += amplitide;
                    amplitide *= persistence;
                }

                result/max_amplitude
            },
            TerrainLayer::NoiseRidge { frequency: freq, persistence, octaves } => {
                let mut result = 0.0;
                let mut max_amplitude = 0.0;
                let mut amplitide = 1.0;
                let mut frequency = *freq;
                for _ in 0..*octaves {
                    result += simdnoise::scalar::simplex_3d(dir.x*frequency, dir.y*frequency, dir.z*frequency) * amplitide;
                    frequency *= 2.0;
                    max_amplitude += amplitide;
                    amplitide *= persistence;
                }

                result/max_amplitude
            },
            TerrainLayer::NoiseSimplex => {
                simdnoise::scalar::simplex_3d(dir.x, dir.y, dir.z)
            },
            TerrainLayer::NoiseTurbulence { freq, lacunarity, gain, octaves } => {
                simdnoise::scalar::turbulence_3d(dir.x, dir.y, dir.z, *freq, *lacunarity, *gain, *octaves)
            }
        }
    }

    pub fn compute_height_and_color(&self, dir: &Vector3<f32>) -> (f32, Vector3<f32>) {
        let height = self.compute_height(&dir);
        let color = self.compute_color_from_height(height);
        (height, color)
    }

    fn compute_color_from_height(&self, height: f32) -> Vector3<f32> {
        // TODO: get this mapping from the terrain parameter file
        let mapping = [
            (200.0, Vector3::new(0.0, 0.0, 0.6)),
            (250.0, Vector3::new(0.0, 0.5, 0.0)),
            (700.0, Vector3::new(0.0, 0.5, 0.0)),
            (1000.0, Vector3::new(0.5, 0.5, 0.5)),
        ];

        let mut entry_low = &mapping[0];
        let mut entry_hi = &mapping[0];
        for entry in &mapping {
            entry_low = entry;

            let (entry_height, _entry_color) = entry;
            if height < *entry_height {
                break;
            }

            entry_hi = entry;
        }

        let (height_low, color_low) = entry_low;
        let (height_hi, color_hi) = entry_hi;

        let a = f32::max(0.0, f32::min(1.0, (height - height_low) / (height_hi - height_low)));

        let color = (1.0 - a) * color_low + a * color_hi;

        color
    }
}