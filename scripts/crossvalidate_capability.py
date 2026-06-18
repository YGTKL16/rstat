#!/usr/bin/env python3
import json
import os
import numpy as np
from scipy import stats

def main():
    # Deterministic data: 15 values
    data = [2.1, 2.5, 2.3, 2.0, 2.4, 2.2, 2.5, 2.8, 2.6, 2.2, 2.3, 2.1, 2.4, 2.6, 2.5]
    
    subgroup_size = 3
    lsl = 1.8
    usl = 3.0
    
    arr = np.array(data, dtype=float)
    mean_val = float(np.mean(arr))
    overall_sigma = float(np.std(arr, ddof=1))
    
    # Calculate R-bar
    n_subgroups = len(data) // subgroup_size
    ranges = []
    for idx in range(n_subgroups):
        chunk = data[idx*subgroup_size : (idx+1)*subgroup_size]
        ranges.append(max(chunk) - min(chunk))
    mean_range = np.mean(ranges)
    
    # d2 constant for subgroup size 3 is 1.693
    d2 = 1.693
    within_sigma = float(mean_range / d2)
    
    # Cp, Cpk, Pp, Ppk
    cp = float((usl - lsl) / (6.0 * within_sigma))
    pp = float((usl - lsl) / (6.0 * overall_sigma))
    
    cpl = (mean_val - lsl) / (3.0 * within_sigma)
    cpu = (usl - mean_val) / (3.0 * within_sigma)
    cpk = float(min(cpl, cpu))
    
    ppl = (mean_val - lsl) / (3.0 * overall_sigma)
    ppu = (usl - mean_val) / (3.0 * overall_sigma)
    ppk = float(min(ppl, ppu))
    
    # PPM calculations using scipy Normal distribution
    norm = stats.norm(loc=mean_val, scale=overall_sigma)
    ppm_lcl = float(norm.cdf(lsl) * 1_000_000.0)
    ppm_usl = float((1.0 - norm.cdf(usl)) * 1_000_000.0)
    ppm_total = ppm_lcl + ppm_usl
    
    # Skewness
    skew_val = float(stats.skew(arr, bias=True))
    normality_warning = bool(abs(skew_val) > 1.0)
    
    expected = {
        "mean": mean_val,
        "overall_sigma": overall_sigma,
        "within_sigma": within_sigma,
        "cp": cp,
        "cpk": cpk,
        "pp": pp,
        "ppk": ppk,
        "ppm_lcl": ppm_lcl,
        "ppm_usl": ppm_usl,
        "ppm_total": ppm_total,
        "skewness": skew_val,
        "normality_warning": normality_warning
    }
    
    # Write CSV
    fixtures_dir = "/home/ygtkula/Masaüstü/Projeler/rstat/crates/rstat-cli/tests/fixtures"
    os.makedirs(fixtures_dir, exist_ok=True)
    
    csv_path = os.path.join(fixtures_dir, "capability_data.csv")
    with open(csv_path, "w") as f:
        f.write("value\n")
        for x in data:
            f.write(f"{x}\n")
            
    # Write JSON
    json_path = os.path.join(fixtures_dir, "capability_expected.json")
    with open(json_path, "w") as f:
        json.dump(expected, f, indent=2)
        
    print(f"Wrote {csv_path} and {json_path}")

if __name__ == "__main__":
    main()
