use std::collections::BTreeMap;
use font8x8::{UnicodeFonts, BASIC_FONTS};








pub struct GlyphCache {
    
    map: BTreeMap<(char, u32), (u32, u32, Vec<u8>)>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self { map: BTreeMap::new() }
    }

    
    pub fn get(&mut self, ch: char, target_h: u32) -> (u32, u32, Vec<u8>) {
        debug_assert!(target_h > 0, "target height must be > 0");

        if let Some(v) = self.map.get(&(ch, target_h)) {
            return (v.0, v.1, v.2.clone());
        }

        
        
        
        let rows: [u8; 8] = BASIC_FONTS.get(ch).unwrap_or([0u8; 8]);

        
        let base_w: u32 = 8;
        let base_h: u32 = 8;
        let target_w: u32 = target_h; 

        
        let mut mask = vec![0u8; (target_w * target_h) as usize];

        for ty in 0..target_h {
            let sy = (ty * base_h) / target_h;
            let row_bits = rows[sy as usize];
            for tx in 0..target_w {
                let sx = (tx * base_w) / target_w;
                
                let on = (row_bits >> sx) & 1;
                mask[(ty * target_w + tx) as usize] = if on == 1 { 255 } else { 0 };
            }
        }

        
        self.map.insert((ch, target_h), (target_w, target_h, mask.clone()));
        (target_w, target_h, mask)
    }
}
