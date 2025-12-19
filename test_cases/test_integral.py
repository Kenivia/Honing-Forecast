# Python 3 — requires only stdlib (cmath, math)

import cmath, math

# parameters
k1 = 100000
k2 = 200000
p1 = 0.7
q1 = 1 - p1
p2 = 0.4
q2 = 1 - p2
t = -4.11593800809674e-7 +  3.154142309803622e-5j  # complex t

# direct mgf and K
M1 = p1 + q1 * cmath.exp(t * k1)
M2 = p2 + q2 * cmath.exp(t * k2)
MS = M1 * M2
K_direct = cmath.log(MS)

# stable (per variable) function
def stable_log_mgf_binary(p, q, k, t):
    u = t.real
    v = t.imag
    alpha0 = math.log(p) if p>0 else float('-inf')
    alpha1 = math.log(q) + u * k
    m = max(alpha0, alpha1)
    b0 = math.exp(alpha0 - m) if p>0 else 0.0
    b1 = math.exp(alpha1 - m) if q>0 else 0.0
    
    tildeS = b0 * cmath.exp(1j * v * 0) + b1 * cmath.exp(1j * v * k)
    print("t",t)
    print("m", m)
    print("sum", tildeS)
    if abs(tildeS) == 0.0:
        return complex(float('-inf'), 0.0)
    return m + cmath.log(tildeS)

L1 = stable_log_mgf_binary(p1, q1, k1, t)
L2 = stable_log_mgf_binary(p2, q2, k2, t)
K_stable = L1 + L2

print("K_direct =", K_direct)
print("K_stable =", K_stable)
print("difference =", K_direct - K_stable)
