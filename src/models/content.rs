use nalgebra::DVector;

#[derive(Debug, Clone)]
pub struct Content {
    pub id: usize,
    pub creator_id: usize,
    pub timestamp: i64,
    pub tags: Vec<String>,
    pub engagement_score: f32,
    pub vector_representation: DVector<f32>,
    pub length: i32,
}
