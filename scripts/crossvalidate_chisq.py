#!/usr/bin/env python3
"""rstat chi-square cross-validation fixture ve expected değer üretici.
scipy referans değerleri üretir ve CSV/JSON fixture dosyalarını yazar.
"""
import json
import os
import numpy as np
from scipy import stats

fixtures_dir = "/home/ygtkula/Masaüstü/Projeler/rstat/crates/rstat-cli/tests/fixtures"
os.makedirs(fixtures_dir, exist_ok=True)

# 1. 2x2 Bağımsızlık Matrisi
matrix_2x2 = np.array([
    [10.0, 20.0],
    [20.0, 30.0]
])
matrix_2x2_path = os.path.join(fixtures_dir, "chisq_independence_2x2.csv")
with open(matrix_2x2_path, "w") as f:
    f.write("col1,col2\n")
    for row in matrix_2x2:
        f.write(f"{row[0]},{row[1]}\n")

# Scipy ile 2x2 Yates'li ve Yates'siz çözümü
res_2x2_yates = stats.chi2_contingency(matrix_2x2, correction=True)
res_2x2_no_yates = stats.chi2_contingency(matrix_2x2, correction=False)

# Cramér's V hesabı (uncorrected chi2 ile)
chi2_uncorr = res_2x2_no_yates.statistic
n_total = matrix_2x2.sum()
v_2x2 = np.sqrt(chi2_uncorr / (n_total * 1))

# 2. 3x3 Bağımsızlık Matrisi
matrix_3x3 = np.array([
    [15.0, 25.0, 30.0],
    [20.0, 10.0, 15.0],
    [35.0, 15.0, 20.0]
])
matrix_3x3_path = os.path.join(fixtures_dir, "chisq_independence_3x3.csv")
with open(matrix_3x3_path, "w") as f:
    f.write("A,B,C\n")
    for row in matrix_3x3:
        f.write(f"{row[0]},{row[1]},{row[2]}\n")

res_3x3 = stats.chi2_contingency(matrix_3x3, correction=False)
v_3x3 = np.sqrt(res_3x3.statistic / (matrix_3x3.sum() * 2))

# 3. Bağımsızlık Uzun Format (Long Format)
# Kategori kombinasyonlarından ham veri seti üretelim
# A-X: 15, A-Y: 25, B-X: 30, B-Y: 10
long_data = []
long_data.extend([("A", "X")] * 15)
long_data.extend([("A", "Y")] * 25)
long_data.extend([("B", "X")] * 30)
long_data.extend([("B", "Y")] * 10)

matrix_long = np.array([
    [15.0, 25.0],
    [30.0, 10.0]
])
res_long_yates = stats.chi2_contingency(matrix_long, correction=True)
res_long_no_yates = stats.chi2_contingency(matrix_long, correction=False)
v_long = np.sqrt(res_long_no_yates.statistic / (matrix_long.sum() * 1))

long_path = os.path.join(fixtures_dir, "chisq_independence_long.csv")
with open(long_path, "w") as f:
    f.write("var1,var2\n")
    for v1, v2 in long_data:
        f.write(f"{v1},{v2}\n")

# 4. Uyum İyiliği Testi (Goodness of Fit)
obs_gof = np.array([20.0, 30.0, 50.0])
exp_gof = np.array([25.0, 25.0, 50.0]) # or proportions [0.25, 0.25, 0.50]
# Normalised / scaled expected for scipy
res_gof = stats.chisquare(obs_gof, f_exp=exp_gof)

gof_path = os.path.join(fixtures_dir, "chisq_gof.csv")
with open(gof_path, "w") as f:
    f.write("observed,expected_prop\n")
    # write observed counts and expected proportions
    props = exp_gof / exp_gof.sum()
    for o, p in zip(obs_gof, props):
        f.write(f"{o},{p}\n")

# low expected warning testi için küçük beklenen değerli veri seti
obs_low = np.array([10.0, 2.0])
exp_low = np.array([10.0, 3.0])
exp_low_scaled = exp_low * (obs_low.sum() / exp_low.sum())
res_low = stats.chisquare(obs_low, f_exp=exp_low_scaled)

# Expected JSON dosyası
expected = {
    "tolerance": 1e-9,
    "independence_2x2_yates": {
        "statistic": float(res_2x2_yates.statistic),
        "p_value": float(res_2x2_yates.pvalue),
        "df": float(res_2x2_yates.dof),
        "cramers_v": float(v_2x2),
        "expected": res_2x2_yates.expected_freq.tolist(),
        "warning_low_expected": bool((res_2x2_yates.expected_freq < 5.0).any())
    },
    "independence_2x2_no_yates": {
        "statistic": float(res_2x2_no_yates.statistic),
        "p_value": float(res_2x2_no_yates.pvalue),
        "df": float(res_2x2_no_yates.dof),
        "cramers_v": float(v_2x2),
        "expected": res_2x2_no_yates.expected_freq.tolist(),
        "warning_low_expected": bool((res_2x2_no_yates.expected_freq < 5.0).any())
    },
    "independence_3x3": {
        "statistic": float(res_3x3.statistic),
        "p_value": float(res_3x3.pvalue),
        "df": float(res_3x3.dof),
        "cramers_v": float(v_3x3),
        "expected": res_3x3.expected_freq.tolist(),
        "warning_low_expected": bool((res_3x3.expected_freq < 5.0).any())
    },
    "independence_long_yates": {
        "statistic": float(res_long_yates.statistic),
        "p_value": float(res_long_yates.pvalue),
        "df": float(res_long_yates.dof),
        "cramers_v": float(v_long),
        "expected": res_long_yates.expected_freq.tolist(),
        "warning_low_expected": bool((res_long_yates.expected_freq < 5.0).any())
    },
    "gof": {
        "statistic": float(res_gof.statistic),
        "p_value": float(res_gof.pvalue),
        "df": float(len(obs_gof) - 1),
        "warning_low_expected": bool((exp_gof < 5.0).any())
    },
    "gof_low": {
        "statistic": float(res_low.statistic),
        "p_value": float(res_low.pvalue),
        "df": float(len(obs_low) - 1),
        "warning_low_expected": True
    }
}

expected_json_path = os.path.join(fixtures_dir, "chisq_expected.json")
with open(expected_json_path, "w") as f:
    json.dump(expected, f, indent=2)

print(f"Fixture CSV dosyaları ve expected JSON dosyası başarıyla üretildi: {fixtures_dir}")
