pub mod error;
pub mod dom_types;
pub mod layout_types;
pub mod net_types;

use dom_types::parse_html_min;
use error::{EngineError, Result};
use layout_types::{build_render_plan, style_tree, LayoutConfig, RenderPlan};

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub scripting_enabled: bool,
    pub sri_by_default: bool,
    pub site_isolation: bool,
    pub telemetry_enabled: bool,
    pub layout: LayoutConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            scripting_enabled: false,
            sri_by_default: true,
            site_isolation: true,
            telemetry_enabled: false,
            layout: LayoutConfig::default(),
        }
    }
}

pub fn first_paint(html: &str, cfg: Option<&EngineConfig>) -> Result<RenderPlan> {
    let cfg = cfg.cloned().unwrap_or_default();

    let dom = parse_html_min(html)?;

    let styled = style_tree(&dom);

    let plan: RenderPlan = build_render_plan(&dom, &styled, &cfg.layout);

    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout_types::PaintCmd;

    #[test]
    fn smoke_first_paint() {
        let html = r#"
        <div style="background:#eeeeee">
            <p style="color:#333333;font-size:18px">Hello Aevrix</p>
            <span>Deterministic First Paint</span>
        </div>
        "#;

        let plan = first_paint(html, None).expect("first paint");
        assert!(!plan.paint_list.is_empty());
        let texts: Vec<_> = plan
            .paint_list
            .iter()
            .filter_map(|p| match p { PaintCmd::DrawText { text, .. } => Some(text.as_str()), _ => None })
            .collect();

        assert_eq!(texts[0], "Hello Aevrix");
        assert_eq!(texts[1], "Deterministic First Paint");
        assert!(matches!(plan.paint_list[0], PaintCmd::FillRect{..}));
    }
}
