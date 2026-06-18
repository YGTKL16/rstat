#!/usr/bin/env python3
import json
from scipy import stats

fixtures = []
# F-dağılımı test vakaları
f_cases = [
    (3.5, 2.0, 10.0),
    (1.0, 1.0, 1.0),
    (0.5, 4.0, 20.0),
    (10.2, 5.0, 12.0),
]
for f, df1, df2 in f_cases:
    p = stats.f.sf(f, df1, df2)
    fixtures.append({
        "type": "F",
        "x": f,
        "df1": df1,
        "df2": df2,
        "p_value": p
    })

# χ²-dağılımı test vakaları
chi_cases = [
    (5.5, 3.0),
    (1.2, 1.0),
    (15.0, 5.0),
    (0.5, 2.0),
]
for chi, df in chi_cases:
    p = stats.chi2.sf(chi, df)
    fixtures.append({
        "type": "Chi2",
        "x": chi,
        "df1": df,
        "p_value": p
    })

print(json.dumps(fixtures, indent=2))
