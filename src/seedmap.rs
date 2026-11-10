//! Seed viewer (`/tools/seed`) : réglages communs à la page et au calcul.
//!
//! Le calcul ne se fait pas sur le serveur : cubiomes (MIT, `vendor/cubiomes`) est compilé en
//! WebAssembly (`public/wasm/cubiomes.wasm`, par `scripts/build-wasm.sh`) et tourne dans le
//! navigateur du visiteur, dans des Web Workers (`public/js/seed-worker.js`). Le serveur ne fait
//! que servir la page et les fichiers.

/// Cases par côté d'une tuile.
pub const TUILE: i32 = 128;
/// Blocs par case possibles (zoom), du plus proche au plus loin.
pub const ECHELLES: [i32; 4] = [4, 16, 64, 256];
/// Blocs par côté d'une région de structures.
pub const REGION: i32 = 2048;

/// Versions proposées : (libellé, identifiant cubiomes), la plus récente d'abord.
pub const VERSIONS: &[(&str, i32)] = &[
    ("1.21.4+", 28),
    ("1.21.2 – 1.21.3", 27),
    ("1.21 – 1.21.1", 26),
    ("1.20", 25),
    ("1.19.3 – 1.19.4", 24),
    ("1.19 – 1.19.2", 23),
    ("1.18", 22),
    ("1.17", 21),
    ("1.16.2 – 1.16.5", 20),
    ("1.16.1", 19),
    ("1.15", 18),
    ("1.14", 17),
    ("1.13", 16),
    ("1.12", 15),
    ("1.11", 14),
    ("1.10", 13),
    ("1.9", 12),
    ("1.8", 11),
    ("1.7", 10),
    ("1.6", 9),
    ("1.5", 8),
    ("1.4", 7),
    ("1.3", 6),
    ("1.2", 5),
    ("1.1", 4),
    ("1.0", 3),
];

/// Une sorte de structure : (clé, dimension, types cubiomes regroupés).
pub const STRUCTURES: &[(&str, i32, &[i32])] = &[
    ("village", 0, &[5]),
    ("outpost", 0, &[10]),
    ("mansion", 0, &[9]),
    ("monument", 0, &[8]),
    ("temple", 0, &[1, 2, 3, 4]),
    ("ruined_portal", 0, &[11]),
    ("ancient_city", 0, &[13]),
    ("trial_chambers", 0, &[24]),
    ("fortress", -1, &[18]),
    ("bastion", -1, &[19]),
    ("ruined_portal", -1, &[12]),
    ("end_city", 1, &[20]),
];

#[cfg(test)]
mod tests {
    /// Le module WebAssembly est bien là, et exporte ce que seed-worker.js appelle.
    #[test]
    fn wasm_present_et_complet() {
        let wasm = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/public/wasm/cubiomes.wasm"))
            .expect("public/wasm/cubiomes.wasm manque : lancer scripts/build-wasm.sh");
        assert_eq!(&wasm[..4], b"\0asm");
        for nom in ["misty_alloc", "misty_biomes", "misty_couleurs", "misty_nom_biome", "misty_spawn", "misty_structures", "_initialize"] {
            assert!(wasm.windows(nom.len()).any(|w| w == nom.as_bytes()), "export {nom} absent");
        }
    }
}
