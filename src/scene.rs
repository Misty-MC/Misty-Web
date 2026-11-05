//! Paysage nocturne en pixel art (montagnes, brume, lune, sapins), généré en SVG au démarrage.
//! Le tirage est déterministe : la même graine donne toujours le même paysage.

use std::fmt::Write;

const W: i32 = 1600;
const H: i32 = 440;
const P: i32 = 8; // taille d'un « pixel »
const COLS: i32 = W / P;

pub struct SceneOptions {
    pub seed: u64,
    /// Position de la lune, en fraction de la largeur.
    pub moon_x: f32,
    /// Rayon de la lune, en pixels de la scène.
    pub moon_r: i32,
}

/// Générateur pseudo-aléatoire minimal (xorshift64*).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn float(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo) as u64) as i32
    }
}

/// Hauteur de crête (en lignes de pixels depuis le haut) pour chaque colonne.
fn ridge(rng: &mut Rng, base: f32, amp: f32, freq: f32) -> Vec<i32> {
    let (p1, p2, p3) = (rng.float() * 6.3, rng.float() * 6.3, rng.float() * 6.3);
    let mut jitter = 0i32;
    (0..COLS)
        .map(|c| {
            let x = c as f32 * freq;
            let wave =
                0.55 * (x + p1).sin() + 0.3 * (x * 2.3 + p2).sin() + 0.15 * (x * 5.1 + p3).sin();
            if rng.float() < 0.35 {
                jitter = (jitter + rng.range(-1, 2)).clamp(-1, 1);
            }
            (base - amp * wave).round() as i32 + jitter
        })
        .collect()
}

/// Remplit sous la crête, en fusionnant les colonnes voisines de même hauteur.
fn ridge_path(heights: &[i32]) -> String {
    let mut d = String::new();
    let mut c = 0;
    while c < heights.len() {
        let h = heights[c];
        let start = c;
        while c < heights.len() && heights[c] == h {
            c += 1;
        }
        let _ = write!(
            d,
            "M{} {}h{}V{H}H{}z",
            start as i32 * P,
            h * P,
            (c - start) as i32 * P,
            start as i32 * P
        );
    }
    d
}

fn rect(d: &mut String, x: i32, y: i32, w: i32, h: i32) {
    let _ = write!(d, "M{x} {y}h{w}v{h}h-{w}z");
}

/// Une nappe de brume en pixels : quelques rangées empilées qui rétrécissent vers le haut.
fn puff_path(rng: &mut Rng) -> String {
    let mut d = String::new();
    let rows = rng.range(3, 6);
    let base = rng.range(16, 44);
    let (mut left, mut right) = (0, base);
    for r in 0..rows {
        rect(&mut d, left * P, (rows - 1 - r) * P, (right - left) * P, P);
        left += rng.range(1, 5);
        right -= rng.range(1, 5);
        if right - left < 4 {
            break;
        }
    }
    d
}

/// Ajoute `n` nappes de brume qui dérivent de gauche à droite, entre les lignes `rows`.
/// Chaque nappe a sa vitesse et son décalage, pour que le mouvement ne se répète pas.
fn puffs(svg: &mut String, rng: &mut Rng, n: i32, rows: (i32, i32), opacity: f32) {
    for _ in 0..n {
        let y = rng.range(rows.0, rows.1) * P;
        // Position de repos (visible si les animations sont coupées). L'animation la décale de
        // -2000 à +2000 : quelle que soit cette position, la nappe entre et sort hors champ.
        let x = rng.range(0, W / P) * P;
        let dur = rng.range(150, 320);
        let delay = rng.range(0, dur);
        let op = opacity * (0.7 + rng.float() * 0.6);
        let _ = write!(
            svg,
            r##"<g transform="translate({x} {y})"><path class="puff" style="animation-duration:{dur}s;animation-delay:-{delay}s" d="{}" fill="#d6e4ff" opacity="{op:.3}"/></g>"##,
            puff_path(rng)
        );
    }
}

/// Bande de brume seule (sans paysage), posée entre les sections et au-dessus du pied de page.
pub fn fog_strip(seed: u64) -> String {
    const SH: i32 = 160;
    let mut rng = Rng(seed | 1);
    let mut svg = format!(
        r##"<svg class="fogstrip" viewBox="0 0 {W} {SH}" preserveAspectRatio="xMidYMid slice" shape-rendering="crispEdges" aria-hidden="true" focusable="false">"##
    );
    puffs(&mut svg, &mut rng, 16, (2, SH / P - 6), 0.1);
    svg.push_str("</svg>");
    svg
}

pub fn render(opt: &SceneOptions) -> String {
    let mut rng = Rng(opt.seed | 1);
    let mut svg = String::with_capacity(32 * 1024);
    let _ = write!(
        svg,
        r##"<svg class="scene" viewBox="0 0 {W} {H}" preserveAspectRatio="xMidYMax slice" shape-rendering="crispEdges" aria-hidden="true" focusable="false">
<defs>
<radialGradient id="glow"><stop offset="0" stop-color="#bcd4ff" stop-opacity=".45"/><stop offset=".4" stop-color="#8fb3ff" stop-opacity=".12"/><stop offset="1" stop-color="#8fb3ff" stop-opacity="0"/></radialGradient>
<linearGradient id="fog" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#cfe0ff" stop-opacity="0"/><stop offset=".5" stop-color="#cfe0ff" stop-opacity=".16"/><stop offset="1" stop-color="#cfe0ff" stop-opacity="0"/></linearGradient>
<linearGradient id="fade" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#0a0f1d" stop-opacity="0"/><stop offset="1" stop-color="#0a0f1d"/></linearGradient>
</defs>
"##
    );

    // Étoiles
    let (mut stars, mut twinkle) = (String::new(), String::new());
    for _ in 0..90 {
        let (x, y) = (rng.range(0, COLS) * P, rng.range(0, 26) * P);
        let s = if rng.float() < 0.2 { P / 2 } else { P / 4 };
        rect(
            if rng.float() < 0.3 {
                &mut twinkle
            } else {
                &mut stars
            },
            x,
            y,
            s,
            s,
        );
    }
    let _ = write!(
        svg,
        r##"<path d="{stars}" fill="#dbe7ff" opacity=".55"/><path class="twinkle" d="{twinkle}" fill="#ffffff"/>"##
    );

    // Lune
    let (mx, my, r) = ((opt.moon_x * W as f32) as i32 / P * P, 13 * P, opt.moon_r);
    let _ = write!(
        svg,
        r##"<g class="moon-glow"><circle cx="{mx}" cy="{my}" r="{}" fill="url(#glow)"/></g>"##,
        r * 5
    );
    let (mut moon, mut craters) = (String::new(), String::new());
    let cells = r / P;
    for cy in -cells..=cells {
        for cx in -cells..=cells {
            if cx * cx + cy * cy <= cells * cells {
                rect(&mut moon, mx + cx * P - P / 2, my + cy * P - P / 2, P, P);
                // Quelques cratères discrets, à des positions relatives au rayon.
                let crater = [(-2, -1), (-1, -1), (-2, 0), (2, 2), (1, -3), (3, -1)]
                    .iter()
                    .any(|&(x, y)| cx == x * cells / 5 && cy == y * cells / 5);
                if crater {
                    rect(&mut craters, mx + cx * P - P / 2, my + cy * P - P / 2, P, P);
                }
            }
        }
    }
    let _ = write!(
        svg,
        r##"<g class="moon"><path d="{moon}" fill="#eaf1ff"/><path d="{craters}" fill="#c4d4f2"/></g>"##
    );

    // Crêtes de l'arrière vers l'avant, séparées par des bancs de brume qui dérivent.
    let layers: [(f32, f32, f32, &str); 4] = [
        (30.0, 8.0, 0.045, "#24345a"),
        (37.0, 7.0, 0.07, "#1a2748"),
        (44.0, 5.0, 0.09, "#121c36"),
        (50.0, 3.0, 0.12, "#0d1528"),
    ];
    for (i, (base, amp, freq, color)) in layers.iter().enumerate() {
        let heights = ridge(&mut rng, *base, *amp, *freq);
        let _ = write!(
            svg,
            r##"<path d="{}" fill="{color}"/>"##,
            ridge_path(&heights)
        );

        // Sapins sur les deux crêtes les plus proches.
        if i >= 2 {
            let mut trees = String::new();
            let mut c = rng.range(2, 10);
            while c < COLS - 3 {
                let top = heights[c as usize];
                let tall = rng.range(3, if i == 3 { 7 } else { 5 });
                for row in 0..tall {
                    let half = (row + 1) / 2;
                    rect(
                        &mut trees,
                        (c - half) * P,
                        (top - tall + row) * P,
                        (half * 2 + 1) * P,
                        P,
                    );
                }
                c += rng.range(4, 16);
            }
            let _ = write!(svg, r##"<path d="{trees}" fill="{color}"/>"##);
        }

        // Nappes de brume qui traversent la scène, de plus en plus visibles vers l'avant.
        let row = *base as i32;
        puffs(
            &mut svg,
            &mut rng,
            3 + i as i32,
            (row - 6, row + 2),
            0.05 + 0.025 * i as f32,
        );

        if i < 3 {
            let y = ((base + 2.0) as i32 - 2) * P;
            let _ = write!(
                svg,
                r##"<rect class="fog fog{i}" x="-200" y="{y}" width="{}" height="{}" fill="url(#fog)"/>"##,
                W + 400,
                8 * P
            );
        }
    }

    let _ = write!(
        svg,
        r##"<rect x="0" y="{}" width="{W}" height="{}" fill="url(#fade)"/></svg>"##,
        H - 12 * P,
        12 * P
    );
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_valid() {
        let opt = SceneOptions {
            seed: 42,
            moon_x: 0.7,
            moon_r: 40,
        };
        let a = render(&opt);
        assert_eq!(a, render(&opt));
        assert!(a.starts_with("<svg") && a.ends_with("</svg>"));
        assert!(a.contains(r#"class="moon""#) && a.contains(r#"class="puff""#));
        let f = fog_strip(3);
        assert_eq!(f, fog_strip(3));
        assert!(f.starts_with("<svg") && f.ends_with("</svg>") && f.contains("puff"));
    }
}
