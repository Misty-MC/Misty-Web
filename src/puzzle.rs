//! Captcha « puzzle » : une image (le paysage de nuit du site et le logo de Misty) avec un trou,
//! et la pièce qui y entre. Le membre fait glisser la pièce jusqu'au trou.
//!
//! Les deux images sont des PNG dessinés ici : la position du trou n'apparaît nulle part dans la
//! page. La réponse est jugée sur la position finale (à quelques pixels près) et sur le geste
//! (durée, nombre de points, vitesse irrégulière) pour écarter les robots les plus simples.

use crate::scene::{self, SceneOptions};
use resvg::tiny_skia::{
    self, FillRule, FilterQuality, Paint, PathBuilder, Pattern, Pixmap, SpreadMode, Stroke, Transform,
};
use std::sync::{Arc, LazyLock};

/// Taille de l'image de fond.
pub const W: u32 = 340;
pub const H: u32 = 200;
/// Corps carré de la pièce et rayon de son ergot (à droite).
const CORPS: f32 = 48.0;
const ERGOT: f32 = 9.0;
/// Largeur et hauteur totales de la pièce.
pub const PIECE_W: u32 = (CORPS + ERGOT) as u32;
pub const PIECE_H: u32 = CORPS as u32;
/// Tolérance sur la position finale, en pixels de l'image.
const TOLERANCE: f32 = 6.0;

static LOGO: &[u8] = include_bytes!("../public/assets/icone.png");

static OPTIONS: LazyLock<Arc<resvg::usvg::Options<'static>>> = LazyLock::new(|| Arc::new(resvg::usvg::Options::default()));

/// Position du trou pour une graine : x à droite de la pièce de départ, y libre.
pub fn position(seed: u64) -> (u32, u32) {
    let x_min = PIECE_W + 30;
    let x_max = W - PIECE_W - 8;
    let y_max = H - PIECE_H - 14;
    let x = x_min + (seed % (x_max - x_min) as u64) as u32;
    let y = 14 + ((seed >> 16) % (y_max - 14) as u64) as u32;
    (x, y)
}

/// Forme de la pièce (carré + ergot), placée en (x, y).
fn forme(x: f32, y: f32) -> Option<tiny_skia::Path> {
    // un seul contour : carré, avec un demi-cercle qui dépasse au milieu du côté droit
    let (cx, cy, r) = (x + CORPS, y + CORPS / 2.0, ERGOT);
    let k = 0.552_284_8 * r;
    let mut pb = PathBuilder::new();
    pb.move_to(x, y);
    pb.line_to(cx, y);
    pb.line_to(cx, cy - r);
    pb.cubic_to(cx + k, cy - r, cx + r, cy - k, cx + r, cy);
    pb.cubic_to(cx + r, cy + k, cx + k, cy + r, cx, cy + r);
    pb.line_to(cx, y + CORPS);
    pb.line_to(x, y + CORPS);
    pb.close();
    pb.finish()
}

/// Fond : le paysage (graine différente à chaque défi) et le logo à un endroit au hasard.
fn fond(seed: u64) -> Option<Pixmap> {
    use base64::Engine;
    let scene = scene::render(&SceneOptions { seed, moon_x: 0.15 + (seed % 70) as f32 / 100.0, moon_r: 36 });
    let logo = base64::engine::general_purpose::STANDARD.encode(LOGO);
    // logo près du trou : la pièce en contient toujours un morceau (bien visible)
    let taille = 104 + (seed >> 8) % 30;
    let (hx, hy) = position(seed);
    let (cx, cy) = (hx as i64 + 24 + ((seed >> 24) % 50) as i64 - 25, hy as i64 + 24 + ((seed >> 32) % 40) as i64 - 20);
    let lx = (cx - taille as i64 / 2).clamp(PIECE_W as i64 + 10, (W - 4) as i64 - taille as i64) as u64;
    let ly = (cy - taille as i64 / 2).clamp(4, (H - 4) as i64 - taille as i64) as u64;
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">
<rect width="{W}" height="{H}" fill="#0b1224"/>
<svg width="{W}" height="{H}" viewBox="0 0 1600 440" preserveAspectRatio="xMidYMax slice">{}</svg>
<circle cx="{}" cy="{}" r="{}" fill="#6f9cff" opacity=".18"/>
<image href="data:image/png;base64,{logo}" x="{lx}" y="{ly}" width="{taille}" height="{taille}"/>
</svg>"##,
        interieur(&scene),
        lx + taille / 2,
        ly + taille / 2,
        taille / 2 + 18,
    );
    let tree = resvg::usvg::Tree::from_str(&svg, &OPTIONS).ok()?;
    let mut pixmap = Pixmap::new(W, H)?;
    resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
    Some(pixmap)
}

/// Le SVG du paysage a sa propre balise `<svg>` : on garde son contenu.
fn interieur(s: &str) -> &str {
    let debut = s.find('>').map(|i| i + 1).unwrap_or(0);
    let fin = s.rfind("</svg>").unwrap_or(s.len());
    &s[debut..fin]
}

/// Image de fond avec le trou (assombri, bord clair).
pub fn image_fond(seed: u64) -> Option<Vec<u8>> {
    let mut p = fond(seed)?;
    let (x, y) = position(seed);
    let trou = forme(x as f32, y as f32)?;
    let mut ombre = Paint::default();
    ombre.set_color_rgba8(4, 8, 20, 185);
    ombre.anti_alias = true;
    p.fill_path(&trou, &ombre, FillRule::Winding, Transform::identity(), None);
    let mut bord = Paint::default();
    bord.set_color_rgba8(255, 255, 255, 120);
    bord.anti_alias = true;
    p.stroke_path(&trou, &bord, &Stroke { width: 1.5, ..Default::default() }, Transform::identity(), None);
    p.encode_png().ok()
}

/// La pièce : le morceau d'image découpé à l'emplacement du trou, avec un bord blanc.
pub fn image_piece(seed: u64) -> Option<Vec<u8>> {
    let src = fond(seed)?;
    let (x, y) = position(seed);
    // marge pour le bord et l'ombre
    let m = 3.0;
    let mut p = Pixmap::new(PIECE_W + 6, PIECE_H + 6)?;
    let piece = forme(m, m)?;
    let mut ombre = Paint::default();
    ombre.set_color_rgba8(0, 0, 0, 90);
    ombre.anti_alias = true;
    p.fill_path(&piece, &ombre, FillRule::Winding, Transform::from_translate(1.5, 2.0), None);
    let paint = Paint {
        shader: Pattern::new(
            src.as_ref(),
            SpreadMode::Pad,
            FilterQuality::Nearest,
            1.0,
            Transform::from_translate(m - x as f32, m - y as f32),
        ),
        anti_alias: true,
        ..Default::default()
    };
    p.fill_path(&piece, &paint, FillRule::Winding, Transform::identity(), None);
    let mut bord = Paint::default();
    bord.set_color_rgba8(255, 255, 255, 235);
    bord.anti_alias = true;
    p.stroke_path(&piece, &bord, &Stroke { width: 2.0, ..Default::default() }, Transform::identity(), None);
    p.encode_png().ok()
}

/// Le geste ressemble-t-il à celui d'un humain ? `trace` : (temps en ms, x) pendant le glissement.
pub fn geste_humain(trace: &[(f32, f32)]) -> bool {
    if trace.len() < 6 {
        return false;
    }
    let duree = trace.last().map(|p| p.0).unwrap_or(0.0) - trace[0].0;
    if !(250.0..=60_000.0).contains(&duree) {
        return false;
    }
    // vitesse irrégulière : un script avance souvent à pas parfaitement constants
    let pas: Vec<i32> = trace.windows(2).map(|w| ((w[1].1 - w[0].1) * 10.0).round() as i32).collect();
    let mut distincts = pas.clone();
    distincts.sort_unstable();
    distincts.dedup();
    distincts.len() >= 3
}

/// Position finale assez proche du trou ?
pub fn bonne_position(seed: u64, x: f32) -> bool {
    (x - position(seed).0 as f32).abs() <= TOLERANCE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_et_positions() {
        for seed in [1u64, 42, 0xdead_beef, u64::MAX] {
            let (x, y) = position(seed);
            assert!(x > PIECE_W + 20 && x + PIECE_W <= W && y + PIECE_H <= H);
        }
        let fond = image_fond(7).unwrap();
        let piece = image_piece(7).unwrap();
        assert!(fond.starts_with(b"\x89PNG") && piece.starts_with(b"\x89PNG"));
        let (x, _) = position(7);
        assert!(bonne_position(7, x as f32 + 4.0));
        assert!(!bonne_position(7, x as f32 + 12.0));
    }

    #[test]
    fn gestes() {
        // robot : pas constants
        let robot: Vec<(f32, f32)> = (0..20).map(|i| (i as f32 * 20.0, i as f32 * 10.0)).collect();
        assert!(!geste_humain(&robot));
        // trop rapide
        assert!(!geste_humain(&[(0.0, 0.0), (5.0, 50.0), (10.0, 90.0), (15.0, 120.0), (20.0, 150.0), (25.0, 160.0)]));
        // humain : accélère puis ralentit
        let humain = [(0.0, 0.0), (40.0, 3.0), (80.0, 15.0), (120.0, 40.0), (180.0, 80.0), (240.0, 110.0), (300.0, 124.0), (380.0, 130.0), (450.0, 131.5)];
        assert!(geste_humain(&humain));
    }

    /// `PREVIEW_DIR=… cargo test apercu_puzzle -- --ignored`
    #[test]
    #[ignore]
    fn apercu_puzzle() {
        let dir = std::env::var("PREVIEW_DIR").unwrap();
        for seed in [11u64, 0x1234_5678_9abc] {
            std::fs::write(format!("{dir}/fond-{seed}.png"), image_fond(seed).unwrap()).unwrap();
            std::fs::write(format!("{dir}/piece-{seed}.png"), image_piece(seed).unwrap()).unwrap();
        }
    }
}
