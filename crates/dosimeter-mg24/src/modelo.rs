// GERADO por python/treina_gmm.py e python/treina_regressao.py — NÃO EDITAR À MÃO.
//
// Coeficientes do dosímetro: densidade (GMM) e regressão do nível.
// Consumidos por tests/ida_e_volta.rs e, no dispositivo, por src/gmm.rs / src/regress.rs.

pub const N_COMP: usize = 2;
pub const N_BANDAS_M: usize = 12;

pub const GMM_PESOS: [f32; N_COMP] = [5.0000000e-01f32, 5.0000000e-01f32];

pub const GMM_MEDIAS: [[f32; N_BANDAS_M]; N_COMP] = [
    [
        -4.5038666e+01f32,
        -4.4484840e+01f32,
        -4.4062374e+01f32,
        -4.3241722e+01f32,
        -4.2641850e+01f32,
        -4.2311058e+01f32,
        -4.1701881e+01f32,
        -4.1247726e+01f32,
        -4.0695614e+01f32,
        -4.0055904e+01f32,
        -3.9521290e+01f32,
        -3.9079166e+01f32,
    ],
    [
        -3.7979145e+01f32,
        -3.6291538e+01f32,
        -3.3929905e+01f32,
        -3.9667957e+01f32,
        -4.6118332e+01f32,
        -5.0398403e+01f32,
        -5.4034245e+01f32,
        -5.7181427e+01f32,
        -5.9882317e+01f32,
        -6.2186913e+01f32,
        -6.3928383e+01f32,
        -6.4623207e+01f32,
    ],
];

pub const GMM_VARS: [[f32; N_BANDAS_M]; N_COMP] = [
    [
        1.4604131e+00f32,
        1.3333013e+00f32,
        1.3560396e+00f32,
        1.4677536e+00f32,
        1.3275073e+00f32,
        1.4678267e+00f32,
        1.5614944e+00f32,
        1.3658742e+00f32,
        1.4888006e+00f32,
        1.3738908e+00f32,
        1.4435873e+00f32,
        1.5833089e+00f32,
    ],
    [
        1.0522285e+01f32,
        1.2231813e+01f32,
        1.1307120e+01f32,
        1.1398597e+01f32,
        1.1024020e+01f32,
        1.2871595e+01f32,
        1.3522217e+01f32,
        1.2106066e+01f32,
        1.3116759e+01f32,
        1.1801123e+01f32,
        1.1941499e+01f32,
        1.1436303e+01f32,
    ],
];

// --- regressão (acrescentado por python/treina_regressao.py) ---
pub const REG_ALPHA: f32 = 1.0000000e+00f32;
pub const REG_SIGMA2: f32 = 1.0000000e+00f32;
pub const REG_M: [f32; 2] = [2.1073041e+00f32, 9.6060580e-01f32];
pub const REG_A_INV: [[f32; 2]; 2] = [
    [2.0295620e-01f32, -3.7295851e-03f32],
    [-3.7295851e-03f32, 6.9102512e-05f32],
];
