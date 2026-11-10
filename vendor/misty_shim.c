// Petite couche d'accès à cubiomes (MIT, voir cubiomes/LICENSE) pour le seed viewer du site de Misty.
// Compilée en WebAssembly par scripts/build-wasm.sh : le calcul se fait dans le navigateur du
// visiteur (public/js/seed-worker.js), pas sur le serveur.
#include "cubiomes/generator.h"
#include "cubiomes/finders.h"
#include "cubiomes/util.h"
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define EXPORT __attribute__((visibility("default")))

// Le générateur prêt pour la dernière (version, dimension, seed) : le préparer coûte cher en 1.18+,
// et une carte demande des dizaines de tuiles de la même seed.
static Generator g;
static int g_mc = -1, g_dim = 99;
static uint64_t g_seed;

static Generator *generateur(int mc, int dim, int64_t seed)
{
    if (mc != g_mc) {
        setupGenerator(&g, mc, 0);
        g_mc = mc;
        g_dim = 99;
    }
    if (dim != g_dim || (uint64_t)seed != g_seed) {
        applySeed(&g, dim, (uint64_t)seed);
        g_dim = dim;
        g_seed = (uint64_t)seed;
    }
    return &g;
}

EXPORT void *misty_alloc(int taille) { return malloc((size_t)taille); }
EXPORT void misty_free(void *p) { free(p); }

// Biomes d'une zone : sx*sz cases de `scale` blocs, la case (0,0) étant en (x, z) (en cases).
// Un octet par case (identifiant du biome). Renvoie 0 si tout va bien.
EXPORT int misty_biomes(int mc, int dim, int64_t seed, int scale, int x, int z, int sx, int sz, uint8_t *sortie)
{
    Generator *gen = generateur(mc, dim, seed);
    Range r;
    r.scale = scale;
    r.x = x; r.z = z; r.sx = sx; r.sz = sz;
    r.y = dim == 0 ? (scale == 1 ? 63 : 63 / 4) : 0;
    r.sy = 1;
    // genBiomes a besoin d'un tampon de travail plus grand que sx*sz : on passe par allocCache
    int *cache = allocCache(gen, r);
    if (!cache) return 1;
    int erreur = genBiomes(gen, cache, r);
    if (!erreur)
        for (int i = 0; i < sx * sz; i++) sortie[i] = (uint8_t)(cache[i] < 0 ? 0 : cache[i] > 255 ? 255 : cache[i]);
    free(cache);
    return erreur;
}

// Couleurs des biomes : 256 × (r, g, b).
EXPORT void misty_couleurs(unsigned char *sortie768)
{
    unsigned char c[256][3];
    initBiomeColors(c);
    for (int i = 0; i < 256; i++) { sortie768[i*3] = c[i][0]; sortie768[i*3+1] = c[i][1]; sortie768[i*3+2] = c[i][2]; }
}

EXPORT const char *misty_nom_biome(int mc, int id) { return biome2str(mc, id); }

EXPORT void misty_spawn(int mc, int64_t seed, int *x, int *z)
{
    Pos p = estimateSpawn(generateur(mc, 0, seed), NULL);
    *x = p.x; *z = p.z;
}

// Structures d'un type dans le rectangle de blocs [x0,x1] x [z0,z1] ; renvoie leur nombre (au plus `max`).
EXPORT int misty_structures(int mc, int dim, int64_t seed, int type, int x0, int z0, int x1, int z1, int *sortie, int max)
{
    StructureConfig sc;
    if (!getStructureConfig(type, mc, &sc)) return 0;
    if (sc.dim != dim) return 0;
    Generator *gen = generateur(mc, dim, seed);
    int taille = sc.regionSize * 16;
    int rx0 = (int)((x0 >= 0 ? x0 : x0 - taille + 1) / taille) - 1, rx1 = (int)((x1 >= 0 ? x1 : x1 - taille + 1) / taille) + 1;
    int rz0 = (int)((z0 >= 0 ? z0 : z0 - taille + 1) / taille) - 1, rz1 = (int)((z1 >= 0 ? z1 : z1 - taille + 1) / taille) + 1;
    int n = 0;
    for (int rz = rz0; rz <= rz1; rz++)
        for (int rx = rx0; rx <= rx1; rx++) {
            Pos p;
            if (!getStructurePos(type, mc, (uint64_t)seed, rx, rz, &p)) continue;
            if (p.x < x0 || p.x > x1 || p.z < z0 || p.z > z1) continue;
            if (!isViableStructurePos(type, gen, p.x, p.z, 0)) continue;
            if (n < max) { sortie[n*2] = p.x; sortie[n*2+1] = p.z; n++; }
        }
    return n;
}
