#!/usr/bin/env python3
"""rstat t-test cross-validation fixture üretici.
scipy referans değerleri üretir — Rust testleri bunlarla 1e-10 toleransta doğrulanır.
Kullanım: python3 scripts/crossvalidate_ttest.py > scripts/ttest_fixtures.json
"""
import json
import numpy as np
from scipy import stats


def one_sample(x, mu0, alt):
    x = np.asarray(x, dtype=float)
    n = x.size
    res = stats.ttest_1samp(x, popmean=mu0, alternative=alt)
    df = n - 1
    s = x.std(ddof=1)
    se = s / np.sqrt(n)
    tcrit = stats.t.ppf(0.975, df)
    mean = x.mean()
    ci = [mean - tcrit * se, mean + tcrit * se]
    d = (mean - mu0) / s
    return dict(
        kind="one", mu0=mu0, alt=alt, n=int(n),
        statistic=float(res.statistic), df=float(df),
        p_value=float(res.pvalue), mean=float(mean), std=float(s),
        mean_diff=float(mean - mu0), ci=[float(ci[0]), float(ci[1])],
        cohens_d=float(d),
    )


def two_sample(a, b, equal_var, alt):
    a = np.asarray(a, float)
    b = np.asarray(b, float)
    res = stats.ttest_ind(a, b, equal_var=equal_var, alternative=alt)
    n1, n2 = a.size, b.size
    v1, v2 = a.var(ddof=1), b.var(ddof=1)
    m1, m2 = a.mean(), b.mean()
    if equal_var:
        df = n1 + n2 - 2
        sp2 = ((n1 - 1) * v1 + (n2 - 1) * v2) / df
        se = np.sqrt(sp2 * (1 / n1 + 1 / n2))
    else:
        e1, e2 = v1 / n1, v2 / n2
        df = (e1 + e2) ** 2 / (e1 ** 2 / (n1 - 1) + e2 ** 2 / (n2 - 1))
        se = np.sqrt(e1 + e2)
    tcrit = stats.t.ppf(0.975, df)
    diff = m1 - m2
    ci = [diff - tcrit * se, diff + tcrit * se]
    sp = np.sqrt(((n1 - 1) * v1 + (n2 - 1) * v2) / (n1 + n2 - 2))
    d = diff / sp
    return dict(
        kind="two", method=("pooled" if equal_var else "welch"), alt=alt,
        n1=int(n1), n2=int(n2), statistic=float(res.statistic), df=float(df),
        p_value=float(res.pvalue), mean1=float(m1), mean2=float(m2),
        std1=float(np.sqrt(v1)), std2=float(np.sqrt(v2)),
        mean_diff=float(diff), ci=[float(ci[0]), float(ci[1])],
        cohens_d=float(d),
    )


def paired(a, b, alt):
    a = np.asarray(a, float)
    b = np.asarray(b, float)
    res = stats.ttest_rel(a, b, alternative=alt)
    d = a - b
    n = d.size
    df = n - 1
    sd = d.std(ddof=1)
    se = sd / np.sqrt(n)
    dbar = d.mean()
    tcrit = stats.t.ppf(0.975, df)
    ci = [dbar - tcrit * se, dbar + tcrit * se]
    cohens_d = dbar / sd
    return dict(
        kind="paired", alt=alt, n=int(n), statistic=float(res.statistic),
        df=float(df), p_value=float(res.pvalue), mean_diff=float(dbar),
        std_diff=float(sd), ci=[float(ci[0]), float(ci[1])],
        cohens_d=float(cohens_d),
    )


# Sabit, deterministik veri setleri (seed yok — tekrarlanabilir)
X  = [5.1, 4.9, 6.2, 5.5, 5.0, 4.8, 6.1, 5.3, 5.7, 4.95]
A  = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]
B  = [1.0, 2.0, 3.0, 3.5, 4.0, 4.0, 5.0, 8.0]
P1 = [10.2, 11.5, 9.8, 12.1, 10.9, 11.3]
P2 = [9.9, 11.0, 9.5, 11.8, 10.5, 11.1]

fixtures = []
for alt in ("two-sided", "less", "greater"):
    fixtures.append(one_sample(X, 5.0, alt))
    fixtures.append(two_sample(A, B, equal_var=False, alt=alt))
    fixtures.append(two_sample(A, B, equal_var=True, alt=alt))
    fixtures.append(paired(P1, P2, alt))

print(json.dumps({
    "tolerance": 1e-10,
    "datasets": {"X": X, "A": A, "B": B, "P1": P1, "P2": P2},
    "fixtures": fixtures,
}, indent=2))
