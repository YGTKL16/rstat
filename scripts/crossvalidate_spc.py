#!/usr/bin/env python3
import json
import os
import numpy as np

CONSTANTS = {
    2: {"a2": 1.880, "d3": 0.000, "d4": 3.267, "d2": 1.128},
    3: {"a2": 1.023, "d3": 0.000, "d4": 2.574, "d2": 1.693},
    4: {"a2": 0.729, "d3": 0.000, "d4": 2.282, "d2": 2.059},
    5: {"a2": 0.577, "d3": 0.000, "d4": 2.114, "d2": 2.326},
    6: {"a2": 0.483, "d3": 0.000, "d4": 2.004, "d2": 2.534},
    7: {"a2": 0.419, "d3": 0.076, "d4": 1.924, "d2": 2.704},
    8: {"a2": 0.373, "d3": 0.136, "d4": 1.864, "d2": 2.847},
    9: {"a2": 0.337, "d3": 0.184, "d4": 1.816, "d2": 2.970},
    10: {"a2": 0.308, "d3": 0.223, "d4": 1.777, "d2": 3.078},
}

def check_rules(means, lcl, ucl, center):
    violations = []
    for i in range(len(means)):
        # Rule 1
        if means[i] < lcl or means[i] > ucl:
            violations.append({
                "rule_id": 1,
                "subgroup_index": i,
                "description": f"Kural 1 İhlali: Nokta ({means[i]:.6f}) kontrol limitlerinin dışında [LCL: {lcl:.6f}, UCL: {ucl:.6f}]"
            })
        
        # Rule 2: 9 consecutive on one side
        if i >= 8:
            slice_vals = means[i-8:i+1]
            all_above = all(x > center for x in slice_vals)
            all_below = all(x < center for x in slice_vals)
            if all_above or all_below:
                side = "üstünde" if all_above else "altında"
                violations.append({
                    "rule_id": 2,
                    "subgroup_index": i,
                    "description": f"Kural 2 İhlali: 9 ardışık nokta merkez çizgisinin aynı tarafında ({side})"
                })
        
        # Rule 3: 6 consecutive increasing or decreasing
        if i >= 5:
            slice_vals = means[i-5:i+1]
            increasing = all(slice_vals[j] > slice_vals[j-1] for j in range(1, 6))
            decreasing = all(slice_vals[j] < slice_vals[j-1] for j in range(1, 6))
            if increasing or decreasing:
                dir_str = "artıyor" if increasing else "azalıyor"
                violations.append({
                    "rule_id": 3,
                    "subgroup_index": i,
                    "description": f"Kural 3 İhlali: 6 ardışık nokta sürekli {dir_str}"
                })
        
        # Rule 4: 14 points alternating
        if i >= 13:
            diffs = [means[j] - means[j-1] for j in range(i-12, i+1)]
            alternating = True
            for j in range(1, len(diffs)):
                if diffs[j] * diffs[j-1] >= 0.0:
                    alternating = False
                    break
            if alternating:
                violations.append({
                    "rule_id": 4,
                    "subgroup_index": i,
                    "description": "Kural 4 İhlali: 14 ardışık nokta inişli çıkışlı dalgalanıyor"
                })
    return violations

def main():
    # Deterministic data: 20 values, subgroup_size = 5 -> 4 subgroups
    data = [10.2, 10.4, 9.8, 10.0, 9.9,
            10.1, 10.3, 9.7, 10.0, 10.1,
            10.2, 10.4, 9.8, 10.0, 9.9,
            25.0, 10.0, 10.0, 10.0, 10.0]
    
    subgroup_size = 5
    n_subgroups = len(data) // subgroup_size
    
    subgroups = []
    sum_means = 0.0
    sum_ranges = 0.0
    
    for idx in range(n_subgroups):
        chunk = data[idx*subgroup_size : (idx+1)*subgroup_size]
        m = float(np.mean(chunk))
        r = float(np.max(chunk) - np.min(chunk))
        subgroups.append({
            "index": idx,
            "values": [float(x) for x in chunk],
            "mean": m,
            "range": r
        })
        sum_means += m
        sum_ranges += r
        
    grand_mean = sum_means / n_subgroups
    mean_range = sum_ranges / n_subgroups
    
    c = CONSTANTS[subgroup_size]
    estimated_sigma = mean_range / c["d2"]
    
    xbar_lcl = grand_mean - c["a2"] * mean_range
    xbar_ucl = grand_mean + c["a2"] * mean_range
    
    r_lcl = c["d3"] * mean_range
    r_ucl = c["d4"] * mean_range
    
    means = [s["mean"] for s in subgroups]
    violations = check_rules(means, xbar_lcl, xbar_ucl, grand_mean)
    
    expected = {
        "subgroup_size": subgroup_size,
        "grand_mean": float(grand_mean),
        "mean_range": float(mean_range),
        "estimated_sigma": float(estimated_sigma),
        "xbar_limits": {
            "lcl": float(xbar_lcl),
            "cl": float(grand_mean),
            "ucl": float(xbar_ucl)
        },
        "r_limits": {
            "lcl": float(r_lcl),
            "cl": float(mean_range),
            "ucl": float(r_ucl)
        },
        "subgroups": subgroups,
        "violations": violations
    }
    
    # Write CSV
    fixtures_dir = "/home/ygtkula/Masaüstü/Projeler/rstat/crates/rstat-cli/tests/fixtures"
    os.makedirs(fixtures_dir, exist_ok=True)
    
    csv_path = os.path.join(fixtures_dir, "spc_data.csv")
    with open(csv_path, "w") as f:
        f.write("value\n")
        for x in data:
            f.write(f"{x}\n")
            
    # Write JSON
    json_path = os.path.join(fixtures_dir, "spc_expected.json")
    with open(json_path, "w") as f:
        json.dump(expected, f, indent=2)
        
    print(f"Wrote {csv_path} and {json_path}")

if __name__ == "__main__":
    main()
