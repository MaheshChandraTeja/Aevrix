use std::collections::BTreeMap;





#[derive(Default)]
pub struct FontAtlas {
    cache: BTreeMap<(u32, char), f32>, 
}

impl FontAtlas {
    pub fn new() -> Self { Self { cache: BTreeMap::new() } }

    #[inline]
    pub fn advance(&mut self, ch: char, size_px: f32) -> f32 {
        let size_key = (size_px * 100.0).round() as u32;
        if let Some(v) = self.cache.get(&(size_key, ch)) {
            return *v;
        }
        let k = classify(ch);
        let adv = k * size_px;
        self.cache.insert((size_key, ch), adv);
        adv
    }

    
    pub fn measure(&mut self, s: &str, size_px: f32) -> f32 {
        s.chars().map(|c| self.advance(c, size_px)).sum()
    }
}

#[inline]
fn classify(c: char) -> f32 {
    
    
    
    
    
    if c.is_ascii_whitespace() {
        0.33
    } else if c.is_ascii_punctuation() {
        0.28
    } else if c.is_ascii() {
        0.56
    } else {
        0.60
    }
}
