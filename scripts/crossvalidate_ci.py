#!/usr/bin/env python3
import json
import numpy as np
from scipy import stats

# Ortalama ve varyans için veri seti
data = [1.5, 2.0, 2.5, 3.0, 3.5, 4.0]
n = len(data)
m = np.mean(data)
s = np.std(data, ddof=1)
v = np.var(data, ddof=1)
sem = stats.sem(data)

# %95 Ortalama GA
mean_ci = stats.t.interval(0.95, df=n-1, loc=m, scale=sem)

# %95 Varyans GA
chi2_low = stats.chi2.ppf(0.025, n-1)
chi2_high = stats.chi2.ppf(0.975, n-1)
var_ci = [(n-1)*v/chi2_high, (n-1)*v/chi2_low]

# Oran için veri seti: başarı = 4, deneme = 10
p = 0.4
n_prop = 10
z = stats.norm.ppf(0.975)

# Wald oran GA
wald_se = np.sqrt(p * (1 - p) / n_prop)
wald_ci = [p - z * wald_se, p + z * wald_se]

# Wilson oran GA
denom = 1 + z**2 / n_prop
center = (p + z**2 / (2 * n_prop)) / denom
spread = z * np.sqrt(p * (1 - p) / n_prop + z**2 / (4 * n_prop**2)) / denom
wilson_ci = [center - spread, center + spread]

results = {
    "mean_ci": {
        "data": data,
        "ci_95": [float(mean_ci[0]), float(mean_ci[1])]
    },
    "variance_ci": {
        "data": data,
        "ci_95": [float(var_ci[0]), float(var_ci[1])]
    },
    "proportion_ci": {
        "successes": 4,
        "trials": 10,
        "wald_95": [float(wald_ci[0]), float(wald_ci[1])],
        "wilson_95": [float(wilson_ci[0]), float(wilson_ci[1])]
    }
}

print(json.dumps(results, indent=2))
