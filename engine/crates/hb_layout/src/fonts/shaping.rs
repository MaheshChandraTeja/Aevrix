use crate::fonts::atlas::FontAtlas;



pub fn measure_text(s: &str, size_px: f32, atlas: &mut FontAtlas) -> f32 {
    atlas.measure(s, size_px)
}
