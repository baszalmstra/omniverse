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

#[derive(Serialize, Deserialize)]
pub enum TerrainCombinator {
    Add,
    Multiply
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub enum TerrainFunction {
    Children { op: TerrainCombinator, children: Vec<TerrainLayer> },
    Constant(f32),
    Noise(NoiseFunction)
}

#[derive(Serialize, Deserialize)]
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
}